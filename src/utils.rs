use comfy_table::presets::ASCII_FULL;
use comfy_table::Table;

use crate::types::SearchResult;

pub fn search_results_to_table(search_results: &Vec<SearchResult>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(ASCII_FULL)
        .set_header(vec!["SlNo", "Title", "Provider"]);

    for (i, result) in search_results.iter().enumerate() {
        table.add_row(vec![
            (i + 1).clone().to_string(),
            result.title.clone(),
            result.provider.clone(),
        ]);
    }
    return table;
}
