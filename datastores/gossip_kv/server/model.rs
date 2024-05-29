use hydroflow::lattices::map_union::MapUnionHashMap;
use hydroflow::lattices::set_union::SetUnionHashSet;
use hydroflow::lattices::{DomPair, Max};

/// Primary key for entries in a table.
pub type RowKey = String;

/// Value stored in a table. Modelled as a timestamped set of strings.
///
/// Each value is timestamped with the time at which it was last updated. Concurrent updates at
/// the same timestamp are stored as a set.
pub type RowValue<C> = DomPair<C, SetUnionHashSet<String>>;

/// A map from row keys to values in a table.
pub type Table<C> = MapUnionHashMap<RowKey, RowValue<C>>;

/// Name of a table in the data store.
pub type TableName = String;

/// A map from table names to tables.
pub type TableMap<C> = MapUnionHashMap<TableName, Table<C>>;

/// Timestamps used in the model.
// TODO: This will be updated to use a more sophisticated clock type with https://github.com/hydro-project/hydroflow/issues/1207.
pub type Clock = Max<usize>;

/// TableMap element to upsert a row in an existing TableMap.
///
/// Merge this into an existing TableMap to upsert a row in a table. If the table does not exist,
/// it gets created. There's no explicit "create table" operation.
///
/// Parameters:
/// - `row_ts`: New timestamp of the row being upserted.
/// - `table_name`: Name of the table.
/// - `key`: Primary key of the row.
/// - `val`: Row value.
pub fn upsert_row<C>(row_ts: C, table_name: TableName, key: RowKey, val: String) -> TableMap<C> {
    let value: RowValue<C> = RowValue::new_from(row_ts, SetUnionHashSet::new_from([val]));
    let row: Table<C> = Table::new_from([(key, value)]);
    TableMap::new_from([(table_name, row)])
}

/// TableMap element to delete a row from an existing TableMap.
///
/// Merge this into an existing TableMap to delete a row from a table.
///
/// Parameters:
/// - `row_ts`: New timestamp of the row being deleted.
/// - `table_name`: Name of the table.
/// - `key`: Primary key of the row.
pub fn delete_row<C>(row_ts: C, table_name: TableName, key: RowKey) -> TableMap<C> {
    let value: RowValue<C> = RowValue::new_from(row_ts, SetUnionHashSet::new_from([]));
    let row: Table<C> = Table::new_from([(key, value)]);
    TableMap::new_from([(table_name, row)])
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use hydroflow::lattices::map_union::MapUnionHashMap;
    use hydroflow::lattices::Merge;

    use crate::model::{delete_row, upsert_row, Clock, RowKey, TableMap, TableName};

    #[test]
    fn test_table_map() {
        let mut table_map: TableMap<Clock> = MapUnionHashMap::default();

        let first_tick: Clock = Clock::new(0);
        let second_tick: Clock = Clock::new(1);

        let members_table = TableName::from("members");
        let key_1 = RowKey::from("key1");
        let value_1: String = "value1".to_string();

        // Table starts out empty.
        assert_eq!(table_map.as_reveal_ref().len(), 0, "Expected empty table.");

        let insert = upsert_row(
            first_tick,
            members_table.clone(),
            key_1.clone(),
            value_1.clone(),
        );
        Merge::merge(&mut table_map, insert);
        {
            let table = table_map.as_reveal_ref().get(&members_table).unwrap();

            let row = table.as_reveal_ref().get(&key_1);
            assert!(row.is_some(), "Row should exist");
            assert_eq!(
                *row.unwrap().as_reveal_ref().0,
                first_tick,
                "Unexpected row timestamp"
            );

            let value = row.unwrap().as_reveal_ref().1.as_reveal_ref();
            assert_eq!(
                value,
                &HashSet::from([value_1.to_string()]),
                "Unexpected row value"
            );
        }

        let delete_row = delete_row(second_tick, members_table.clone(), key_1.to_string());
        Merge::merge(&mut table_map, delete_row);
        {
            let table = table_map.as_reveal_ref().get(&members_table).unwrap();

            // Deletion in this case leaves a "tombstone"
            let row = table.as_reveal_ref().get(&key_1);

            assert!(row.is_some(), "Row should exist");
            assert_eq!(
                *row.unwrap().as_reveal_ref().0,
                second_tick,
                "Unexpected row timestamp"
            );

            let value = row.unwrap().as_reveal_ref().1.as_reveal_ref();
            assert_eq!(value, &HashSet::from([]), "Row should be empty");
        }
    }
}
