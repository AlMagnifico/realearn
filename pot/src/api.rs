use crate::plugins::ProductKind;
use crate::provider_database::{
    DatabaseId, FIL_HAS_PREVIEW_TRUE, FIL_IS_FAVORITE_TRUE, FIL_IS_USER_PRESET_FALSE,
    FIL_IS_USER_PRESET_TRUE,
};
use crate::{FilterItem, Preset};
use enum_iterator::IntoEnumIterator;
use enum_map::EnumMap;
use enumset::EnumSet;
use once_cell::sync::Lazy;
use realearn_api::persistence::PotFilterKind;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

/// An ID for uniquely identifying a preset along with its corresponding database.
///
/// This ID is stable only at runtime and only until the database is refreshed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PresetId {
    pub database_id: DatabaseId,
    pub preset_id: InnerPresetId,
}

impl PresetId {
    pub fn new(database_id: DatabaseId, preset_id: InnerPresetId) -> Self {
        Self {
            database_id,
            preset_id,
        }
    }
}

/// An ID for uniquely identifying a preset within a certain database.
///
/// This ID is stable only at runtime and only until the database is refreshed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct InnerPresetId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct FilterItemId(pub Option<Fil>);

impl FilterItemId {
    pub const NONE: Self = Self(None);
}

/// Filter value.
///
/// These can be understood as possible types of a filter item kind. Not all types make sense
/// for a particular filter item kind.
///
/// Many of these types are not suitable for persistence because their values are not stable.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Fil {
    /// A typical integer filter value to refer to a filter item in a specific Komplete database.
    ///
    /// This needs a pot filter item kind and a specific Komplete database to make full sense.
    /// The integers are not suited for being persisted because different Komplete scans can yield
    /// different integers! So they should only be used at runtime and translated to something
    /// more stable for persistence.
    Komplete(u32),
    /// Refers to a specific pot database.
    ///
    /// Only valid at runtime, not suitable for persistence.
    Database(DatabaseId),
    /// Refers to something that can be true of false, e.g. "favorite" or "not favorite"
    /// or "available" or "not available".
    ///
    /// Suitable for persistence.
    Boolean(bool),
    /// Refers to a kind of product.
    ProductKind(ProductKind),
    /// Refers to a product.
    ///
    /// Not suitable for persistence because the product IDs are created at runtime.
    Product(ProductId),
}

/// Id for a [`Product`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, derive_more::Display)]
pub struct ProductId(pub u32);

pub type FilterItemCollections = GenericFilterItemCollections<FilterItem>;

#[derive(Debug)]
pub struct GenericFilterItemCollections<T>(EnumMap<PotFilterKind, Vec<T>>);

pub trait HasFilterItemId {
    fn id(&self) -> FilterItemId;
}

impl HasFilterItemId for FilterItem {
    fn id(&self) -> FilterItemId {
        self.id
    }
}

impl<T> Default for GenericFilterItemCollections<T> {
    fn default() -> Self {
        Self(enum_map::enum_map! { _ => vec![] })
    }
}

impl<T> GenericFilterItemCollections<T> {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn get(&self, kind: PotFilterKind) -> &[T] {
        &self.0[kind]
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (PotFilterKind, &mut Vec<T>)> {
        self.0.iter_mut()
    }

    pub fn set(&mut self, kind: PotFilterKind, items: Vec<T>) {
        self.0[kind] = items;
    }

    pub fn extend(&mut self, kind: PotFilterKind, items: impl Iterator<Item = T>) {
        self.0[kind].extend(items);
    }

    pub fn are_filled_already(&self) -> bool {
        // Just take any of of the constant filters that should be filled.
        !self.get(PotFilterKind::IsFavorite).is_empty()
    }
}

impl<T> IntoIterator for GenericFilterItemCollections<T> {
    type Item = (PotFilterKind, Vec<T>);
    type IntoIter = <EnumMap<PotFilterKind, Vec<T>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: HasFilterItemId> GenericFilterItemCollections<T> {
    pub fn narrow_down(&mut self, kind: PotFilterKind, includes: &HashSet<FilterItemId>) {
        self.0[kind].retain(|item| includes.contains(&item.id()))
    }
}

/// `Some` means a filter is set (can also be the `<None>` filter).
/// `None` means no filter is set (`<Any>`).
pub type OptFilter = Option<FilterItemId>;

#[derive(Copy, Clone, Debug, Default)]
pub struct Filters(EnumMap<PotFilterKind, OptFilter>);

impl Filters {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn wants_preview(&self) -> Option<bool> {
        if let Some(FilterItemId(Some(fil))) = self.get(PotFilterKind::HasPreview) {
            Some(fil == FIL_HAS_PREVIEW_TRUE)
        } else {
            None
        }
    }

    pub fn database_matches(&self, db_id: DatabaseId) -> bool {
        self.matches(PotFilterKind::Database, Fil::Database(db_id))
    }

    pub fn wants_user_presets_only(&self) -> bool {
        self.wants_only(PotFilterKind::IsUser, FIL_IS_USER_PRESET_TRUE)
    }

    pub fn wants_factory_presets_only(&self) -> bool {
        self.wants_only(PotFilterKind::IsUser, FIL_IS_USER_PRESET_FALSE)
    }

    pub fn wants_favorites_only(&self) -> bool {
        self.wants_only(PotFilterKind::IsFavorite, FIL_IS_FAVORITE_TRUE)
    }

    pub fn any_unsupported_filter_is_set_to_concrete_value(
        &self,
        supported_advanced_kinds: EnumSet<PotFilterKind>,
    ) -> bool {
        let supported_kinds = supported_advanced_kinds.union(PotFilterKind::core_kinds());
        supported_kinds
            .complement()
            .iter()
            .any(|k| self.is_set_to_concrete_value(k))
    }

    /// To be used with filter kinds where `<None>` is **not** a valid filter value. In this case,
    /// <None> is considered an invalid value and it never matches (no reason to panic but almost).
    pub fn matches(&self, kind: PotFilterKind, fil: Fil) -> bool {
        match self.get(kind) {
            None => true,
            Some(FilterItemId(None)) => false,
            Some(FilterItemId(Some(wanted_fil))) => fil == wanted_fil,
        }
    }

    pub fn favorite_matches(
        &self,
        favorites: &HashSet<InnerPresetId>,
        preset_id: InnerPresetId,
    ) -> bool {
        match self.get(PotFilterKind::IsFavorite) {
            None => true,
            Some(FilterItemId(None)) => false,
            Some(FilterItemId(Some(fil))) => {
                if fil == FIL_IS_FAVORITE_TRUE {
                    favorites.contains(&preset_id)
                } else {
                    !favorites.contains(&preset_id)
                }
            }
        }
    }

    /// To be used with filter kinds where `<None>` is a valid filter value.
    pub fn matches_optional(&self, kind: PotFilterKind, fil: Option<Fil>) -> bool {
        match self.get(kind) {
            // <Any>
            None => true,
            // <None> or a specific value
            Some(FilterItemId(wanted_fil)) => fil == wanted_fil,
        }
    }

    fn wants_only(&self, kind: PotFilterKind, fil: Fil) -> bool {
        self.get(kind) == Some(FilterItemId(Some(fil)))
    }

    /// Returns `false` if set to `None`
    pub fn is_set_to_concrete_value(&self, kind: PotFilterKind) -> bool {
        matches!(self.0[kind], Some(FilterItemId(Some(_))))
    }

    pub fn get(&self, kind: PotFilterKind) -> OptFilter {
        self.0[kind]
    }

    pub fn get_ref(&self, kind: PotFilterKind) -> &OptFilter {
        &self.0[kind]
    }

    pub fn set(&mut self, kind: PotFilterKind, value: OptFilter) {
        self.0[kind] = value;
    }

    pub fn effective_sub_bank(&self) -> &OptFilter {
        self.effective_sub_item(PotFilterKind::Bank, PotFilterKind::SubBank)
    }

    pub fn clear_excluded_ones(&mut self, exclude_list: &PotFilterExcludes) {
        for kind in PotFilterKind::into_enum_iter() {
            if let Some(id) = self.0[kind] {
                if exclude_list.contains(kind, id) {
                    self.0[kind] = None;
                }
            }
        }
    }

    pub fn clear_if_not_available_anymore(
        &mut self,
        affected_kinds: EnumSet<PotFilterKind>,
        collections: &FilterItemCollections,
    ) {
        for kind in affected_kinds {
            if let Some(id) = self.0[kind] {
                let valid_items = collections.get(kind);
                if !valid_items.iter().any(|item| item.id == id) {
                    self.0[kind] = None;
                }
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (PotFilterKind, OptFilter)> {
        self.0.into_iter()
    }

    pub fn effective_sub_category(&self) -> &OptFilter {
        self.effective_sub_item(PotFilterKind::Category, PotFilterKind::SubCategory)
    }

    pub fn clear_this_and_dependent_filters(&mut self, kind: PotFilterKind) {
        self.set(kind, None);
        for dependent_kind in kind.dependent_kinds() {
            self.set(dependent_kind, None);
        }
    }

    fn effective_sub_item(
        &self,
        parent_kind: PotFilterKind,
        sub_kind: PotFilterKind,
    ) -> &OptFilter {
        let category = &self.0[parent_kind];
        if category == &Some(FilterItemId::NONE) {
            category
        } else {
            &self.0[sub_kind]
        }
    }
}

#[derive(Debug, Default)]
pub struct PotFavorites {
    favorites: HashMap<DatabaseId, HashSet<InnerPresetId>>,
}

impl PotFavorites {
    pub fn is_favorite(&self, preset_id: PresetId) -> bool {
        if let Some(db_favorites) = self.favorites.get(&preset_id.database_id) {
            db_favorites.contains(&preset_id.preset_id)
        } else {
            false
        }
    }

    pub fn toggle_favorite(&mut self, preset_id: PresetId) {
        let db_favorites = self.favorites.entry(preset_id.database_id).or_default();
        if db_favorites.contains(&preset_id.preset_id) {
            db_favorites.remove(&preset_id.preset_id);
        } else {
            db_favorites.insert(preset_id.preset_id);
        }
    }

    pub fn db_favorites(&self, db_id: DatabaseId) -> &HashSet<InnerPresetId> {
        static EMPTY_HASH_SET: Lazy<HashSet<InnerPresetId>> = Lazy::new(HashSet::new);
        self.favorites.get(&db_id).unwrap_or(&EMPTY_HASH_SET)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PotFilterExcludes {
    exluded_items: EnumMap<PotFilterKind, HashSet<FilterItemId>>,
}

impl PotFilterExcludes {
    pub fn contains(&self, kind: PotFilterKind, id: FilterItemId) -> bool {
        self.exluded_items[kind].contains(&id)
    }

    pub fn remove(&mut self, kind: PotFilterKind, id: FilterItemId) {
        self.exluded_items[kind].remove(&id);
    }

    pub fn add(&mut self, kind: PotFilterKind, id: FilterItemId) {
        self.exluded_items[kind].insert(id);
    }

    pub fn is_empty(&self, kind: PotFilterKind) -> bool {
        self.exluded_items[kind].is_empty()
    }

    pub fn contains_database(&self, db_id: DatabaseId) -> bool {
        self.contains(
            PotFilterKind::Database,
            FilterItemId(Some(Fil::Database(db_id))),
        )
    }

    pub fn contains_product(&self, product_id: Option<ProductId>) -> bool {
        self.contains(
            PotFilterKind::Bank,
            FilterItemId(product_id.map(Fil::Product)),
        )
    }

    pub fn normal_excludes_by_kind(&self, kind: PotFilterKind) -> impl Iterator<Item = &Fil> + '_ {
        self.exluded_items[kind]
            .iter()
            .filter_map(|id| id.0.as_ref())
    }

    pub fn contains_none(&self, kind: PotFilterKind) -> bool {
        self.exluded_items[kind].contains(&FilterItemId::NONE)
    }
}

#[derive(Debug)]
pub struct CurrentPreset {
    pub preset: Preset,
    pub macro_param_banks: Vec<MacroParamBank>,
}

#[derive(Debug)]
pub struct MacroParamBank {
    params: Vec<MacroParam>,
}

impl MacroParamBank {
    pub fn new(params: Vec<MacroParam>) -> Self {
        Self { params }
    }

    pub fn name(&self) -> String {
        let mut name = String::with_capacity(32);
        for p in &self.params {
            if !p.section_name.is_empty() {
                if !name.is_empty() {
                    name += " / ";
                }
                name += &p.section_name;
            }
        }
        name
    }

    pub fn params(&self) -> &[MacroParam] {
        &self.params
    }

    pub fn find_macro_param_at(&self, slot_index: u32) -> Option<&MacroParam> {
        self.params.get(slot_index as usize)
    }

    pub fn param_count(&self) -> u32 {
        self.params.len() as _
    }
}

#[derive(Clone, Debug)]
pub struct MacroParam {
    pub name: String,
    pub section_name: String,
    pub param_index: Option<u32>,
}

impl CurrentPreset {
    pub fn preset(&self) -> &Preset {
        &self.preset
    }

    pub fn find_macro_param_bank_at(&self, bank_index: u32) -> Option<&MacroParamBank> {
        self.macro_param_banks.get(bank_index as usize)
    }

    pub fn find_macro_param_at(&self, slot_index: u32) -> Option<&MacroParam> {
        let bank_index = slot_index / 8;
        let bank_slot_index = slot_index % 8;
        self.find_bank_macro_param_at(bank_index, bank_slot_index)
    }

    pub fn find_bank_macro_param_at(
        &self,
        bank_index: u32,
        bank_slot_index: u32,
    ) -> Option<&MacroParam> {
        self.macro_param_banks
            .get(bank_index as usize)?
            .find_macro_param_at(bank_slot_index)
    }

    pub fn macro_param_bank_count(&self) -> u32 {
        self.macro_param_banks.len() as _
    }

    pub fn has_params(&self) -> bool {
        !self.macro_param_banks.is_empty()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PersistentDatabaseId(String);

impl PersistentDatabaseId {
    pub const fn new(raw_id: String) -> Self {
        PersistentDatabaseId(raw_id)
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PersistentInnerPresetId(String);

impl PersistentInnerPresetId {
    pub fn new(raw_id: String) -> Self {
        Self(raw_id)
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

/// A preset ID that survives restarts and rescans.
///
/// It's more expensive to clone, hash etc. than [`PresetId`]. That's why we should only use it
/// for purposes where persistence matters, e.g. when saving favorites, currently selected preset
/// or for associating preview files.
///
/// The schema is `<DATABASE_ID>|<INNER_PRESET_ID>`. The pipe character `|` can also be used within
/// the inner preset ID, so in order to extract the database ID, it's important to split at the
/// first pipe character and ignore the other ones. In general, it should be avoided to use
/// the pipe character in the database ID or inner preset ID, but it's still important to escape it.
///
/// # Examples
///
/// - `defaults|vst2|1967946098`
/// - `track-templates|Synths/Lead.RTrackTemplate`
/// - `fx-chains|Synths/Sun.RfxChain`
/// - `fx-presets|vst3-Surge XT.ini|My Preset`
/// - `komplete|77c5507f5d0b421ea93eeb4cee4b6f99`
/// - `n98h1f9unp92|maojiao/2023-02-03-ben/2023-02-03-ben.RPP|0FF9F738-7CF6-8A49-9AEA-A9AF26DF9C46`
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PersistentPresetId {
    db_id: PersistentDatabaseId,
    inner_preset_id: PersistentInnerPresetId,
}

impl PersistentPresetId {
    pub fn new(db_id: PersistentDatabaseId, inner_preset_id: PersistentInnerPresetId) -> Self {
        Self {
            db_id,
            inner_preset_id,
        }
    }
}

impl Display for PersistentPresetId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Database IDs ideally shouldn't contain pipe characters, but if they do, we escaped them.
        let escaped_db_id = PipeEscaped(self.db_id.get());
        write!(f, "{escaped_db_id}|{}", self.inner_preset_id.get())
    }
}

impl FromStr for PersistentPresetId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (escaped_db_id, inner_preset_id) = s
            .split_once(unescaped_pipe_pattern())
            .ok_or("no | separator found in persistent preset ID")?;
        // Unescaped pipe character (in the unlikely case that there was any)
        let db_id = unescape_pipes(escaped_db_id);
        let id = Self {
            db_id: PersistentDatabaseId(db_id),
            inner_preset_id: PersistentInnerPresetId(inner_preset_id.to_string()),
        };
        Ok(id)
    }
}

pub fn unescaped_pipe_pattern() -> impl FnMut(char) -> bool {
    unescaped_char_pattern('|')
}

/// A Rust string matching pattern that matches the given character, but only if it's not preceded
/// by a backslash.
fn unescaped_char_pattern(needle: char) -> impl FnMut(char) -> bool {
    let mut prev_char = None;
    move |c: char| {
        let matches = if c == needle {
            prev_char != Some('\\')
        } else {
            false
        };
        prev_char = Some(c);
        matches
    }
}

/// Converts "hello\|fellow" to "hello|fellow"
pub fn unescape_pipes(escaped: &str) -> String {
    escaped.replace(r#"\|"#, "|")
}

pub struct PipeEscaped<'a>(pub &'a str);

impl<'a> Display for PipeEscaped<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for c in self.0.chars() {
            if c == '|' {
                f.write_str(r#"\|"#)?;
            } else {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{PersistentDatabaseId, PersistentInnerPresetId, PersistentPresetId};

    #[test]
    fn format_persistent_preset_id() {
        let id = PersistentPresetId::new(
            PersistentDatabaseId::new("test|hello".into()),
            PersistentInnerPresetId::new("vst2|124135".into()),
        );
        assert_eq!(id.to_string(), r#"test\|hello|vst2|124135"#);
    }

    #[test]
    fn parse_persistent_preset_id() {
        let expression = r#"test\|hello|vst2|124135"#;
        let parsed_id: PersistentPresetId = expression.parse().unwrap();
        let expected_id = PersistentPresetId::new(
            PersistentDatabaseId::new("test|hello".into()),
            PersistentInnerPresetId::new("vst2|124135".into()),
        );
        assert_eq!(parsed_id, expected_id);
    }
}