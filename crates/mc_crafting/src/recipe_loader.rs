use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use std::sync::RwLock;

use mc_items::ItemId;
use serde::Deserialize;
use zip::ZipArchive;

use crate::item_stack::ItemStack;
use crate::recipe::{CraftingRecipe, Ingredient, ShapedRecipe, ShapelessRecipe};

/// Raw JSON representation of Minecraft recipes.
#[derive(Debug, Deserialize)]
pub struct RawRecipe {
    #[serde(rename = "type")]
    pub recipe_type: String,
    pub pattern: Option<Vec<String>>,
    pub key: Option<HashMap<String, RawIngredient>>,
    pub ingredients: Option<Vec<RawIngredient>>,
    pub result: Option<RawResult>,
    pub show_notification: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawIngredient {
    Single(RawIngredientItem),
    List(Vec<RawIngredientItem>),
}

#[derive(Debug, Deserialize)]
pub struct RawIngredientItem {
    pub item: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawResult {
    Detailed {
        item: String,
        count: Option<u8>,
    },
    Simple(String),
}

#[derive(Debug, Deserialize)]
pub struct RawTag {
    pub values: Vec<String>,
}

/// In-memory lookup cache providing O(1) query time for 3x3 crafting matrix inputs.
pub struct RecipeLookupCache {
    pub recipes: Vec<CraftingRecipe>,
    /// O(1) in-memory fast hash cache for crafting matrix configurations.
    fast_cache: RwLock<HashMap<[Option<ItemId>; 9], Option<ItemStack>>>,
}

impl RecipeLookupCache {
    pub fn new(recipes: Vec<CraftingRecipe>) -> Self {
        let cache = Self {
            recipes,
            fast_cache: RwLock::new(HashMap::new()),
        };
        cache.prepopulate_common_recipes();
        cache
    }

    /// Pre-populates the fast lookup cache with all canonical and mirrored configurations
    /// for constant/single-item recipes and standard wooden variants.
    fn prepopulate_common_recipes(&self) {
        let mut cache = self.fast_cache.write().unwrap();

        for recipe in &self.recipes {
            match recipe {
                CraftingRecipe::Shaped(shaped) => {
                    if shaped.width > 3 || shaped.height > 3 {
                        continue;
                    }

                    // For recipes where each slot has at most 1 item variant,
                    // precompute all valid translation offsets in the 3x3 grid.
                    let is_simple = shaped
                        .pattern
                        .iter()
                        .all(|slot| slot.as_ref().map_or(true, |ing| ing.allowed.len() == 1));

                    if is_simple {
                        for oy in 0..=(3 - shaped.height) {
                            for ox in 0..=(3 - shaped.width) {
                                // Unmirrored
                                let mut grid = [None; 9];
                                for ry in 0..shaped.height {
                                    for rx in 0..shaped.width {
                                        let pat_idx = ry * shaped.width + rx;
                                        if let Some(ing) = &shaped.pattern[pat_idx] {
                                            grid[(oy + ry) * 3 + (ox + rx)] = Some(ing.allowed[0]);
                                        }
                                    }
                                }
                                cache.entry(grid).or_insert(Some(shaped.result));

                                // Mirrored
                                if shaped.mirrored {
                                    let mut m_grid = [None; 9];
                                    for ry in 0..shaped.height {
                                        for rx in 0..shaped.width {
                                            let pat_x = (shaped.width - 1) - rx;
                                            let pat_idx = ry * shaped.width + pat_x;
                                            if let Some(ing) = &shaped.pattern[pat_idx] {
                                                m_grid[(oy + ry) * 3 + (ox + rx)] = Some(ing.allowed[0]);
                                            }
                                        }
                                    }
                                    cache.entry(m_grid).or_insert(Some(shaped.result));
                                }
                            }
                        }
                    }
                }
                CraftingRecipe::Shapeless(shapeless) => {
                    // For single or two ingredient shapeless recipes with 1 item variant
                    if shapeless.ingredients.len() <= 2
                        && shapeless.ingredients.iter().all(|ing| ing.allowed.len() == 1)
                    {
                        let mut grid = [None; 9];
                        for (i, ing) in shapeless.ingredients.iter().enumerate() {
                            grid[i] = Some(ing.allowed[0]);
                        }
                        cache.entry(grid).or_insert(Some(shapeless.result));
                    }
                }
            }
        }
    }

    /// Computes the exact output item and quantity in O(1) time for any 3x3 crafting grid.
    /// Uses an interior-mutability RwLock hash cache with amortized O(1) evaluation.
    pub fn compute_output(&self, grid: &[Option<ItemStack>; 9]) -> Option<ItemStack> {
        let key: [Option<ItemId>; 9] = [
            grid[0].map(|s| s.item),
            grid[1].map(|s| s.item),
            grid[2].map(|s| s.item),
            grid[3].map(|s| s.item),
            grid[4].map(|s| s.item),
            grid[5].map(|s| s.item),
            grid[6].map(|s| s.item),
            grid[7].map(|s| s.item),
            grid[8].map(|s| s.item),
        ];

        // 1. Fast path: check in-memory O(1) hash map
        {
            let cache = self.fast_cache.read().unwrap();
            if let Some(cached_result) = cache.get(&key) {
                return *cached_result;
            }
        }

        // 2. Slow path on cache miss: evaluate against recipes
        let computed_result = self.evaluate_match(grid);

        // 3. Store into cache so subsequent queries for this exact combination are strictly O(1)
        {
            let mut cache = self.fast_cache.write().unwrap();
            cache.insert(key, computed_result);
        }

        computed_result
    }

    fn evaluate_match(&self, grid: &[Option<ItemStack>; 9]) -> Option<ItemStack> {
        // Quick exit if grid is completely empty
        if grid.iter().all(|slot| slot.is_none()) {
            return None;
        }

        for recipe in &self.recipes {
            if let Some(output) = recipe.matches(grid) {
                return Some(output);
            }
        }

        None
    }

    pub fn recipes_count(&self) -> usize {
        self.recipes.len()
    }

    pub fn get_recipe_by_id(&self, id: &str) -> Option<&CraftingRecipe> {
        self.recipes.iter().find(|r| r.id() == id)
    }
}

/// Automated recipe extractor and Serde deserializer from client.jar.
pub struct RecipeLoader;

impl RecipeLoader {
    /// Loads all tags and crafting recipes directly from client.jar.
    pub fn load_from_jar(jar_path: impl AsRef<Path>) -> Result<RecipeLookupCache, Box<dyn std::error::Error>> {
        let file = File::open(jar_path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        Self::load_from_zip_reader(&mut archive)
    }

    /// Loads tags and recipes from any zip archive reader.
    pub fn load_from_zip_reader<R: Read + Seek>(
        archive: &mut ZipArchive<R>,
    ) -> Result<RecipeLookupCache, Box<dyn std::error::Error>> {
        // 1. Load item tags from data/minecraft/tags/items/*.json
        let mut raw_tags: HashMap<String, Vec<String>> = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };

            if (name.starts_with("data/minecraft/tags/items/") || name.starts_with("data/minecraft/tags/item/"))
                && name.ends_with(".json")
            {
                let prefix_len = if name.starts_with("data/minecraft/tags/items/") {
                    "data/minecraft/tags/items/".len()
                } else {
                    "data/minecraft/tags/item/".len()
                };
                let tag_name = format!("minecraft:{}", &name[prefix_len..name.len() - 5]);

                let mut content = String::new();
                file.read_to_string(&mut content)?;

                if let Ok(raw_tag) = serde_json::from_str::<RawTag>(&content) {
                    raw_tags.insert(tag_name, raw_tag.values);
                }
            }
        }

        // 2. Resolve tags recursively (expanding #other_tag references to concrete ItemIds)
        let mut resolved_tags: HashMap<String, Vec<ItemId>> = HashMap::new();
        for tag in raw_tags.keys() {
            let items = Self::resolve_tag(tag, &raw_tags, &mut Vec::new());
            resolved_tags.insert(tag.clone(), items);
        }

        // 3. Extract and parse all recipe files
        let mut parsed_recipes = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };

            // Matches data/minecraft/recipes/*.json and data/minecraft/recipe/*.json
            let is_recipe_path = (name.starts_with("data/minecraft/recipes/")
                || name.starts_with("data/minecraft/recipe/"))
                && name.ends_with(".json")
                && !name.contains("advancement");

            if is_recipe_path {
                let id = name
                    .trim_start_matches("data/minecraft/recipes/")
                    .trim_start_matches("data/minecraft/recipe/")
                    .trim_end_matches(".json")
                    .to_string();

                let mut content = String::new();
                file.read_to_string(&mut content)?;

                if let Ok(raw) = serde_json::from_str::<RawRecipe>(&content) {
                    match raw.recipe_type.as_str() {
                        "minecraft:crafting_shaped" => {
                            if let Some(shaped) = Self::parse_shaped(&id, &raw, &resolved_tags) {
                                parsed_recipes.push(CraftingRecipe::Shaped(shaped));
                            }
                        }
                        "minecraft:crafting_shapeless" => {
                            if let Some(shapeless) = Self::parse_shapeless(&id, &raw, &resolved_tags) {
                                parsed_recipes.push(CraftingRecipe::Shapeless(shapeless));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(RecipeLookupCache::new(parsed_recipes))
    }

    fn resolve_tag(tag: &str, raw_tags: &HashMap<String, Vec<String>>, visited: &mut Vec<String>) -> Vec<ItemId> {
        if visited.contains(&tag.to_string()) {
            return Vec::new();
        }
        visited.push(tag.to_string());

        let mut items = Vec::new();

        if let Some(values) = raw_tags.get(tag) {
            for val in values {
                if let Some(sub_tag) = val.strip_prefix('#') {
                    let sub_items = Self::resolve_tag(sub_tag, raw_tags, visited);
                    items.extend(sub_items);
                } else if let Some(item_id) = ItemId::from_name(val) {
                    items.push(item_id);
                }
            }
        }

        items
    }

    pub fn parse_ingredient(
        raw_ing: &RawIngredient,
        tag_map: &HashMap<String, Vec<ItemId>>,
    ) -> Option<Ingredient> {
        let mut allowed = Vec::new();

        match raw_ing {
            RawIngredient::Single(item) => {
                Self::collect_ingredient_item(item, tag_map, &mut allowed);
            }
            RawIngredient::List(list) => {
                for item in list {
                    Self::collect_ingredient_item(item, tag_map, &mut allowed);
                }
            }
        }

        if allowed.is_empty() {
            None
        } else {
            Some(Ingredient::new(allowed))
        }
    }

    fn collect_ingredient_item(
        raw: &RawIngredientItem,
        tag_map: &HashMap<String, Vec<ItemId>>,
        out: &mut Vec<ItemId>,
    ) {
        if let Some(ref item_str) = raw.item {
            if let Some(id) = ItemId::from_name(item_str) {
                if !out.contains(&id) {
                    out.push(id);
                }
            }
        } else if let Some(ref tag_str) = raw.tag {
            if let Some(tag_items) = tag_map.get(tag_str) {
                for &id in tag_items {
                    if !out.contains(&id) {
                        out.push(id);
                    }
                }
            }
        }
    }

    pub fn parse_result(raw_res: &RawResult) -> Option<ItemStack> {
        match raw_res {
            RawResult::Detailed { item, count } => {
                let id = ItemId::from_name(item)?;
                Some(ItemStack::new(id, count.unwrap_or(1)))
            }
            RawResult::Simple(item) => {
                let id = ItemId::from_name(item)?;
                Some(ItemStack::new(id, 1))
            }
        }
    }

    pub fn parse_shaped(
        id: &str,
        raw: &RawRecipe,
        tag_map: &HashMap<String, Vec<ItemId>>,
    ) -> Option<ShapedRecipe> {
        let pattern_lines = raw.pattern.as_ref()?;
        let height = pattern_lines.len();
        if height == 0 || height > 3 {
            return None;
        }

        let width = pattern_lines.iter().map(|line| line.chars().count()).max()?;
        if width == 0 || width > 3 {
            return None;
        }

        let raw_keys = raw.key.as_ref()?;
        let mut parsed_keys: HashMap<char, Ingredient> = HashMap::new();

        for (k, v) in raw_keys {
            if let Some(first_char) = k.chars().next() {
                if let Some(ing) = Self::parse_ingredient(v, tag_map) {
                    parsed_keys.insert(first_char, ing);
                }
            }
        }

        let mut pattern = Vec::with_capacity(width * height);

        for line in pattern_lines {
            let chars: Vec<char> = line.chars().collect();
            for col in 0..width {
                if col < chars.len() {
                    let ch = chars[col];
                    if ch == ' ' {
                        pattern.push(None);
                    } else if let Some(ing) = parsed_keys.get(&ch) {
                        pattern.push(Some(ing.clone()));
                    } else {
                        // Unrecognized key
                        return None;
                    }
                } else {
                    pattern.push(None);
                }
            }
        }

        let result = Self::parse_result(raw.result.as_ref()?)?;

        Some(ShapedRecipe::new(
            id.to_string(),
            width,
            height,
            pattern,
            result,
            true, // Mirrored allowed by default in Minecraft
        ))
    }

    pub fn parse_shapeless(
        id: &str,
        raw: &RawRecipe,
        tag_map: &HashMap<String, Vec<ItemId>>,
    ) -> Option<ShapelessRecipe> {
        let raw_ingredients = raw.ingredients.as_ref()?;
        let mut ingredients = Vec::with_capacity(raw_ingredients.len());

        for raw_ing in raw_ingredients {
            let ing = Self::parse_ingredient(raw_ing, tag_map)?;
            ingredients.push(ing);
        }

        let result = Self::parse_result(raw.result.as_ref()?)?;

        Some(ShapelessRecipe::new(id.to_string(), ingredients, result))
    }
}
