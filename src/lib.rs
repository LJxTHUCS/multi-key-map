use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::Hash;

/// A `MultiKeyMap` allows multiple keys to point to a single value.
pub struct MultiKeyMap<K, V> {
    key_map: HashMap<K, usize>,
    values: Vec<V>,
}

impl<K: Eq + Hash + Clone, V> MultiKeyMap<K, V> {
    /// Creates an empty `MultiKeyMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let map: MultiKeyMap<&str, &str> = MultiKeyMap::new();
    /// ```
    pub fn new() -> Self {
        MultiKeyMap {
            key_map: HashMap::new(),
            values: Vec::new(),
        }
    }

    /// Retrieves a reference to a value by its key.
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to lookup.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// assert_eq!(map.get(&"key1"), Some(&"value1"));
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        self.key_map
            .get(key)
            .and_then(|&index| self.values.get(index))
    }

    /// Retrieves a mutable reference to a value by its key.
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to lookup.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// if let Some(value) = map.get_mut(&"key1") {
    ///     *value = "value2";
    /// }
    /// assert_eq!(map.get(&"key1"), Some(&"value2"));
    /// ```
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.key_map
            .get(key)
            .and_then(|index| self.values.get_mut(*index))
    }

    /// Inserts a value with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to insert.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// ```
    pub fn insert(&mut self, key: K, value: V) {
        let index = self.values.len();
        self.values.push(value);
        self.key_map.insert(key, index);
    }

    /// Adds a new alias key for the element at `key`.
    ///
    /// Returns `true` if the alias is successfully added.
    ///
    /// # Arguments
    ///
    /// * `key` - The original key.
    /// * `alias` - The alias key to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// assert!(map.alias(&"key1", "alias1"));
    /// ```
    pub fn alias(&mut self, key: &K, alias: K) -> bool {
        if key == &alias {
            // Do not allow aliasing the same key
            return false;
        }
        if let Some(&index) = self.key_map.get(key) {
            self.key_map.insert(alias, index);
            true
        } else {
            false
        }
    }

    /// Removes an alias key.
    ///
    /// If the last alias is removed, the value is also removed.
    /// Returns `true` if the alias is successfully removed.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias key to remove.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// map.alias(&"key1", "alias1");
    /// assert!(map.remove_alias(&"alias1"));
    /// ```
    pub fn remove_alias(&mut self, alias: &K) -> bool {
        if let Some(&index) = self.key_map.get(alias) {
            let keys_with_index: Vec<K> = self
                .key_map
                .iter()
                .filter_map(|(k, &v)| if v == index { Some(k.clone()) } else { None })
                .collect();
            if keys_with_index.len() > 1 {
                self.key_map.remove(alias);
                true
            } else {
                self.key_map.remove(alias);
                self.values.swap_remove(index);
                // Update the indices for the remaining values
                if index != self.values.len() {
                    let last_index = self.values.len();
                    let last_value_key = self.key_map.iter().find_map(|(k, &v)| {
                        if v == last_index {
                            Some(k.clone())
                        } else {
                            None
                        }
                    });
                    if let Some(key) = last_value_key {
                        self.key_map.insert(key, index);
                    }
                }
                true
            }
        } else {
            false
        }
    }

    /// Removes a value by its key and all its aliases.
    ///
    /// Returns the value if it was present.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// assert_eq!(map.remove(&"key1"), Some("value1"));
    /// assert_eq!(map.get(&"key1"), None);
    /// ```
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(&index) = self.key_map.get(key) {
            let value = self.values.swap_remove(index);
            let keys_to_remove: Vec<K> = self
                .key_map
                .iter()
                .filter_map(|(k, &v)| if v == index { Some(k.clone()) } else { None })
                .collect();

            for k in keys_to_remove {
                self.key_map.remove(&k);
            }
            if index != self.values.len() {
                // Update the index for the value that was moved
                let last_index = self.values.len();
                if let Some(last_key) = self.key_map.iter().find_map(|(k, &v)| {
                    if v == last_index {
                        Some(k.clone())
                    } else {
                        None
                    }
                }) {
                    self.key_map.insert(last_key, index);
                }
            }
            Some(value)
        } else {
            None
        }
    }

    /// Checks if a key exists in the map.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// assert!(map.contains_key(&"key1"));
    /// assert!(!map.contains_key(&"key2"));
    /// ```
    pub fn contains_key(&self, key: &K) -> bool {
        self.key_map.contains_key(key)
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let map: MultiKeyMap<&str, &str> = MultiKeyMap::new();
    /// assert!(map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clears the map, removing all key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.key_map.clear();
        self.values.clear();
    }
}

impl<K: Eq + Hash + Clone + Debug, V: Debug> Debug for MultiKeyMap<K, V> {
    /// Formats the value using the given formatter.
    ///
    /// This trait is used for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to use.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map = MultiKeyMap::new();
    /// map.insert("key1", "value1");
    /// println!("{:?}", map);
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map: HashMap<usize, Vec<&K>> = HashMap::new();
        for (key, &index) in &self.key_map {
            map.entry(index).or_insert_with(Vec::new).push(key);
        }
        let mut debug_struct = f.debug_struct("MultiKeyMap");
        for (index, keys) in map {
            debug_struct.field(&format!("{:?}", keys), &self.values[index]);
        }
        debug_struct.finish()
    }
}

impl<K: Eq + Hash, V: PartialEq> PartialEq for MultiKeyMap<K, V> {
    /// Compares two `MultiKeyMap` instances for equality.
    ///
    /// Two `MultiKeyMap` instances are considered equal if they have the same keys and values,
    /// and each key in one map points to the same value as the corresponding key in the other map.
    ///
    /// # Arguments
    ///
    /// * `other` - The other `MultiKeyMap` to compare against.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut map1 = MultiKeyMap::new();
    /// map1.insert("key1", "value1");
    /// map1.alias(&"key1", "alias1");
    ///
    /// let mut map2 = MultiKeyMap::new();
    /// map2.insert("key1", "value1");
    /// map2.alias(&"key1", "alias1");
    ///
    /// assert_eq!(map1, map2);  // Should be true because both maps have the same keys and values.
    ///
    /// map2.remove_alias(&"alias1");
    /// assert_ne!(map1, map2);  // Should be true because the alias has been removed from map2.
    /// ```
    fn eq(&self, other: &Self) -> bool {
        // Check if both maps have the same number of values
        if self.values.len() != other.values.len() {
            return false;
        }
        // Check if each key in `self` maps to the same value as the corresponding key in `other`
        for (key, &index) in &self.key_map {
            if let Some(&other_index) = other.key_map.get(key) {
                if self.values[index] != other.values[other_index] {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl<K: Eq + Hash, V: PartialEq> Eq for MultiKeyMap<K, V> {}

impl<K: Eq + Hash + Clone + Debug, V: Clone + Debug> Clone for MultiKeyMap<K, V> {
    /// Creates a deep copy of the `MultiKeyMap`.
    ///
    /// This method clones both the `key_map` and the `values` vector to produce a new `MultiKeyMap`
    /// instance that is a copy of the original.
    ///
    /// # Examples
    ///
    /// ```
    /// use multi_key_map::MultiKeyMap;
    /// 
    /// let mut original = MultiKeyMap::new();
    /// original.insert("key1", "value1");
    /// original.alias(&"key1", "alias1");
    ///
    /// let clone = original.clone();
    ///
    /// assert_eq!(original, clone);  // The original and clone should be equal.
    /// ```
    fn clone(&self) -> Self {
        // Clone the values and the key_map
        MultiKeyMap {
            key_map: self.key_map.clone(),
            values: self.values.clone(),
        }
    }
}
