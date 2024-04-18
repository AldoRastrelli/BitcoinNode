pub trait CSVFormat {
    fn get_csv_format(&self) -> Vec<String>;
    fn get_csv_string(&self) -> String {
        let to_csv = self.get_csv_format();
        format!("[{}]", to_csv.join(","))
    }
}
