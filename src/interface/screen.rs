extern crate gtk;

use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::common_traits::csv_format::CSVFormat;
use crate::message_structs::tx_message::TXMessage;
use crate::node::interface::interface_communicator::InterfaceMessages;
use crate::utils::array_tools::u8_array_to_hex_string;
use chrono::Utc;
use glib::{self, Receiver};
use gtk::{
    prelude::*, ButtonsType, ComboBoxText, DialogFlags, Entry, Fixed, ListStore, MenuItem,
    MessageDialog, MessageType, ScrolledWindow, SpinButton, TreePath, TreeView, WindowType, ProgressBar,
};
use gtk::{Adjustment, Builder, Button, Label, Window};
use std::collections::HashMap;
use std::default::Default;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

const HEADER_COLUMNS: usize = 6;
const TRANSACTIONS_COLUMNS: usize = 5;
const MY_CREATED_TRANSACTIONS_COLUMNS: usize = 3;
const BLOCKS_COLUMNS: usize = 4;

pub struct WidgetsGtk {
    tree_view_headers: TreeView,
    list_headers: ListStore,
    tree_view_all_transactions: TreeView,
    list_all_transactions: ListStore,
    tree_view_my_transactions: TreeView,
    list_my_transactions: ListStore,
    tree_view_blocks: TreeView,
    list_blocks: ListStore,
    window_blocks: Window,
    request_button: Button,
    popup: MessageDialog,
    progress_bar: ProgressBar,
}

impl Clone for WidgetsGtk {
    fn clone(&self) -> Self {
        WidgetsGtk {
            tree_view_headers: self.tree_view_headers.clone(),
            list_headers: self.list_headers.clone(),
            tree_view_all_transactions: self.tree_view_all_transactions.clone(),
            list_all_transactions: self.list_all_transactions.clone(),
            tree_view_my_transactions: self.tree_view_my_transactions.clone(),
            list_my_transactions: self.list_my_transactions.clone(),
            tree_view_blocks: self.tree_view_blocks.clone(),
            list_blocks: self.list_blocks.clone(),
            window_blocks: self.window_blocks.clone(),
            request_button: self.request_button.clone(),
            popup: self.popup.clone(),
            progress_bar: self.progress_bar.clone(),
        }
    }
}

/// It builds the screen. It is the main class of the interface.
pub struct Screen {
    builder: Builder,
    widgets: WidgetsGtk,
}

/// It creates the screen with the glade file.
impl Default for Screen {
    fn default() -> Self {
        if let Err(err) = gtk::init() {
            eprintln!("Error initializing GTK: {}", err);
        }
        let builder = Builder::from_file("src/interface/view.glade");
        let widgets = Self::create_widgets(&builder);
        Screen { builder, widgets }
    }
}

impl Screen {
    pub fn create_widgets(builder: &Builder) -> WidgetsGtk {
        let tree_view_headers = Self::set_headers(builder);
        let list_headers = gtk::ListStore::new(&vec![String::static_type(); HEADER_COLUMNS][..]);
        tree_view_headers.set_model(Some(&list_headers));
        let tree_view_all_transactions = Self::set_transactions(builder);
        let list_all_transactions =
            gtk::ListStore::new(&vec![String::static_type(); TRANSACTIONS_COLUMNS][..]);
        tree_view_all_transactions.set_model(Some(&list_all_transactions));
        let tree_view_my_transactions = Self::set_my_transactions(builder);
        let list_my_transactions =
            gtk::ListStore::new(&vec![String::static_type(); MY_CREATED_TRANSACTIONS_COLUMNS][..]);
        tree_view_my_transactions.set_model(Some(&list_my_transactions));
        let tree_view_blocks = Self::set_blocks();
        let list_blocks = gtk::ListStore::new(&vec![String::static_type(); BLOCKS_COLUMNS][..]);
        tree_view_blocks.set_model(Some(&list_blocks));
        let (window_blocks, request_button) = Self::create_blocks_window(&tree_view_blocks);
        let popup = Self::create_popup();
        let progress_bar = Self::set_progress_bar(builder);
        WidgetsGtk {
            tree_view_headers,
            list_headers,
            tree_view_all_transactions,
            list_all_transactions,
            tree_view_my_transactions,
            list_my_transactions,
            tree_view_blocks,
            list_blocks,
            window_blocks,
            request_button,
            popup,
            progress_bar,
        }
    }

    /// Receive the communication vectors and start the main thread function.
    pub fn start(
        sender_to_node: Sender<InterfaceMessages>,
        receiver_from_node: Receiver<InterfaceMessages>,
    ) {
        let mut window = Screen::default();
        window.show_window(sender_to_node, receiver_from_node)
    }

    /// Seters gtk objects ----------------------------

    /// Retrieve the value entered in the SpinButton entry.
    fn _get_value_from_entry(entry: Option<SpinButton>) -> String {
        if let Some(e) = entry {
            e.get_value_as_int().to_string()
        } else {
            String::from("")
        }
    }

    /// Retrieve the value entered in the entry.
    fn get_text_from_entry(entry: Option<Entry>) -> String {
        if let Some(e) = entry {
            String::from(e.get_text())
        } else {
            String::from("")
        }
    }

    /// Set the new value to modify the label.
    fn set_label(builder: &Builder, key: &str, value: &str) {
        let label: Option<Label> = builder.get_object(key);
        if let Some(l) = label {
            l.set_text(value);
        }
    }

    /// Set the columns for the tree view based on their characteristics.
    fn set_columns(tree_view: &TreeView, names: Vec<&str>) {
        for (index, column_name) in names.iter().enumerate() {
            let colum = gtk::TreeViewColumn::new();
            colum.set_title(column_name);
            colum.set_expand(true);
            tree_view.append_column(&colum);
            let cell_renderer = gtk::CellRendererText::new();
            colum.pack_start(&cell_renderer, true);
            colum.add_attribute(&cell_renderer, "text", index as i32);
        }
    }

    /// Set up the scroll windows for the treeview - deprecated.
    fn _set_scrolled_window(&mut self, scrolled_window: Option<ScrolledWindow>) -> ScrolledWindow {
        let mut scrolled: ScrolledWindow =
            ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
        if let Some(s) = scrolled_window {
            s.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
            s.set_hexpand(true);
            s.set_vexpand(true);
            scrolled = s;
        }
        scrolled
    }

    /// Set up the data lists for the treeview - deprecated.
    fn _set_data_lists(&mut self) -> (ScrolledWindow, ScrolledWindow) {
        let window_header: Option<ScrolledWindow> = self.builder.get_object("scroll_window_header");
        let window_blocks: Option<ScrolledWindow> = self.builder.get_object("scroll_window_block");
        let page_headers = self._set_scrolled_window(window_header);
        let page_blocks = self._set_scrolled_window(window_blocks);
        (page_headers, page_blocks)
    }

    /// Set up the tree view headers with their necessary columns.
    fn set_headers(builder: &Builder) -> TreeView {
        let tree_view: Option<TreeView> = builder.get_object("list_headers");
        let tree: TreeView;
        if let Some(t) = tree_view {
            tree = t;
        } else {
            tree = TreeView::new();
        }
        let names = vec![
            "Version",
            "Prev Header",
            "Merkle root",
            "Time To",
            "N° Bits",
            "Nonce",
        ];
        Self::set_columns(&tree, names);
        tree
    }

    /// Set up the tree view blocks with their necessary columns - unused.
    fn set_blocks() -> TreeView {
        let tree = TreeView::new();

        let names = vec!["Id", "Tx Count", "Prev Header", "Merkle root"];
        Self::set_columns(&tree, names);
        tree
    }

    /// Set up the progress bar for incoming blocks
    fn set_progress_bar(builder: &Builder) -> ProgressBar {
        let progress_bar: Option<ProgressBar> = builder.get_object("progress_blocks");
        let bar: ProgressBar;
        if let Some(p) = progress_bar {
            bar = p;
        } else {
            bar = ProgressBar::new();
        }
        bar
    }

    /// Set up the tree view for all transactions with their necessary columns.
    fn set_transactions(builder: &Builder) -> TreeView {
        let tree_view: Option<TreeView> = builder.get_object("list_transactions");
        let tree: TreeView;
        if let Some(t) = tree_view {
            tree = t;
        } else {
            tree = TreeView::new();
        }
        let names = vec!["State", "Belongs User", "Date", "Label", "Amount"];
        Self::set_columns(&tree, names);
        tree
    }

    /// Set up the tree view for my transactions with their necessary columns.
    fn set_my_transactions(builder: &Builder) -> TreeView {
        let tree_view: Option<TreeView> = builder.get_object("list_my_txs");
        let tree: TreeView;
        if let Some(t) = tree_view {
            tree = t;
        } else {
            tree = TreeView::new();
        }
        let names = vec!["Date", "Label", "Amount"];
        Self::set_columns(&tree, names);
        tree
    }

    /// Handle GTK Objects only by actions ----------------------------------

    /// Create the window to add a new wallet.
    fn add_wallet(add_wallet_order: Sender<InterfaceMessages>) {
        let window = Window::new(WindowType::Toplevel);
        window.set_title("New Wallet");
        window.set_default_size(400, 300);
        let fixed = Fixed::new();
        window.add(&fixed);
        let name_entry = Entry::new();
        name_entry.set_visible(true);
        name_entry.set_can_focus(true);
        name_entry.set_widget_name("name_wallet_entry");
        fixed.put(&name_entry, 221, 27);
        let name_label = Label::new(Some("Wallet Name:"));
        name_label.set_widget_name("label_new_wallet");
        fixed.put(&name_label, 61, 35);
        let key_entry = Entry::new();
        key_entry.set_visible(true);
        key_entry.set_can_focus(true);
        key_entry.set_widget_name("key_wallet_entry");
        fixed.put(&key_entry, 221, 103);
        let key_label = Label::new(Some("Private Key:"));
        fixed.put(&key_label, 61, 110);
        let create_button = Button::with_label("Create Wallet");
        create_button.set_visible(true);
        create_button.set_can_focus(true);
        create_button.set_widget_name("button_create_wallet");
        fixed.put(&create_button, 154, 165);
        create_button.connect_clicked(move |_| {
            let wallet_name = name_entry.get_text().to_string();
            let private_key = key_entry.get_text().to_string();
            name_entry.set_text("");
            key_entry.set_text("");
            let message = InterfaceMessages::AddWalletOrder((wallet_name, private_key));
            if add_wallet_order.send(message).is_ok() {}
        });
        window.show_all();
    }

    fn create_popup() -> MessageDialog {
        MessageDialog::new(
            None::<&gtk::Window>,
            DialogFlags::empty(),
            MessageType::Info,
            ButtonsType::Ok,
            "",
        )
    }

    fn create_blocks_window(tree_view: &TreeView) -> (Window, Button) {
        let window = Window::new(WindowType::Toplevel);
        window.set_title("Blocks");
        window.set_default_size(800, 700);
        let fixed = gtk::Fixed::new();
        window.add(&fixed);
        let request_button = Button::with_label("Request");
        request_button.set_sensitive(false);
        request_button.set_size_request(150, 40);
        fixed.put(&request_button, 325, 630);
        let scrolled_window = ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
        scrolled_window.set_size_request(760, 600);
        scrolled_window.set_shadow_type(gtk::ShadowType::In);
        scrolled_window.add(tree_view);
        fixed.put(&scrolled_window, 20, 20);
        (window, request_button)
    }

    /// Handle the necessary signal to close the window when the close button is pressed.
    fn close_button(&self, window: &Window, sender_to_node: Sender<InterfaceMessages>) {
        let button_close: Option<Button> = self.builder.get_object("button_close");
        if let Some(b) = button_close {
            let window_ref = window.clone();

            b.connect_clicked(move |_| {
                if sender_to_node.send(InterfaceMessages::Close(())).is_ok() {}
                window_ref.close();
            });
        }
    }

    fn modify_to_show_my_transaction(transaction: TXMessage) -> Vec<String> {
        let mut values = Vec::new();
        let time = transaction.time;
        let current_datetime = Utc::now().naive_utc();
        let datetime = current_datetime + chrono::Duration::seconds(time as i64);
        let formatted_datetime = datetime.format("%d/%m/%Y %H:%M:%S").to_string();
        values.push(formatted_datetime);
        let label = u8_array_to_hex_string(&transaction.get_id());
        let amounts = transaction.get_output_amounts();
        println!(
            "mi transaccion tiene {:?} y el amount es {:?} y id {:?}",
            transaction,
            amounts,
            transaction.get_id()
        );
        values.push(label);
        let mut amount_values = Vec::new();
        for amount in amounts {
            amount_values.push(amount.to_string());
        }
        let values_to_str = amount_values.join(",");
        values.push(values_to_str);

        values
    }

    fn modify_to_show_transaction(transaction: (bool, TXMessage, bool)) -> Vec<String> {
        let mut values = Vec::new();
        if transaction.0 {
            values.push('\u{2705}'.to_string());
        } else {
            values.push('\u{23F3}'.to_string());
        }
        if transaction.2 {
            values.push('\u{1F64B}'.to_string());
        } else {
            values.push('\u{1F465}'.to_string());
        }
        let time = transaction.1.time;
        let current_datetime = Utc::now().naive_utc();
        let datetime = current_datetime + chrono::Duration::seconds(time as i64);
        let formatted_datetime = datetime.format("%d/%m/%Y %H:%M:%S").to_string();
        values.push(formatted_datetime);
        let label = u8_array_to_hex_string(&transaction.1.get_id());
        values.push(label);
        let amounts = transaction.1.get_output_amounts();
        let mut amount_values = Vec::new();
        for amount in amounts {
            amount_values.push(amount.to_string());
        }
        let values_to_str = amount_values.join(",");
        values.push(values_to_str);
        values
    }

    /// Handle Channels Actions  ------------------------

    /// Handles the signal received from the node that occurs when the interface is opened with pre-existing files.
    fn open_signal(builder: &Builder, open_signal: (Vec<String>, HashMap<String, String>)) {
        let wallets: Option<ComboBoxText> = builder.get_object("wallet_switch");
        if let Some(w) = wallets {
            for name in open_signal.0 {
                w.append_text(&name);
            }
        }
        /*
        for (key, value) in open_signal.1.iter() {
            Self::set_label(builder, key, value);
        }
        */
    }

    /// Sends the signal to the node to create a transaction with the provided data.
    fn send_button(&mut self, send_transaction: Sender<InterfaceMessages>) {
        let button: Option<Button> = self.builder.get_object("button_send");
        let builder_aux = self.builder.clone();
        let sender_clone = send_transaction;
        if let Some(b) = button {
            b.set_sensitive(false);
            b.connect_clicked(move |_| {
                let entry_address: Option<Entry> = builder_aux.get_object("entry_bitcoin_address");
                let entry_label: Option<Entry> = builder_aux.get_object("entry_label");
                let entry_amount: Option<Entry> = builder_aux.get_object("entry_amount");
                let entry_fee: Option<Entry> = builder_aux.get_object("entry_fee");
                let address = Self::get_text_from_entry(entry_address);
                let label = Self::get_text_from_entry(entry_label);
                let amount = Self::get_text_from_entry(entry_amount);
                let fee = Self::get_text_from_entry(entry_fee);
                if let Ok(am) = amount.parse::<i32>() {
                    if let Ok(fe) = fee.parse::<i32>() {
                        if am >= 546 && fe >= 1000 {
                            let message =
                                InterfaceMessages::SendTransaction((address, label, am, fe));
                            if sender_clone.send(message).is_ok() {}
                        }
                    }
                }
            });
        }
    }

    /// Sends a signal to the node indicating a change of wallet, specifying the name of the new wallet to be set as the default.
    fn wallet_switch_button(&mut self, wallet_switch: Sender<InterfaceMessages>) {
        let switch: Option<ComboBoxText> = self.builder.get_object("wallet_switch");
        let button: Option<Button> = self.builder.get_object("button_send");
        if let Some(wallets) = switch {
            let wallet_switcher_clone = wallet_switch;
            wallets.connect_changed(move |wallets| {
                if let Some(name) = wallets.get_active_text() {
                    if let Some(s) = &button {
                        s.set_sensitive(true);
                    }
                    let message = InterfaceMessages::WalletSwitch(name.to_string());
                    if wallet_switcher_clone.send(message).is_ok() {}
                }
            });
        };
    }

    /// Handles the buttons on the taskbar, currently only the one for creating a wallet.
    fn add_wallet_button(&mut self, add_wallet_order: Sender<InterfaceMessages>) {
        let new_button: Option<MenuItem> = self.builder.get_object("new_wallet");
        if let Some(b) = new_button {
            b.connect_activate(move |_| {
                Self::add_wallet(add_wallet_order.clone());
            });
        }
    }

    fn get_selected_row(tree_view: &TreeView, tree_path: &TreePath) -> Vec<String> {
        let mut values = Vec::new();
        if let Some(tree_model) = tree_view.get_model() {
            if let Some(tree_iter) = tree_model.get_iter(tree_path) {
                let column_count = tree_model.get_n_columns();
                for column_id in 0..column_count {
                    if let Ok(Some(value)) =
                        tree_model.get_value(&tree_iter, column_id).get::<String>()
                    {
                        values.push(value);
                    } else {
                        values.push(String::new());
                    }
                }
            }
        }
        values
    }

    fn proof_of_inclusion_button(&mut self, sender_proof_order: Sender<InterfaceMessages>) {
        let inclusion_proof: Option<Button> = self.builder.get_object("inclusion_proof_button");
        let widgets = self.widgets.clone();
        if let Some(b) = inclusion_proof {
            let button = b.clone();
            b.set_sensitive(false);
            let selected_transaction = Arc::new(Mutex::new(Vec::new()));
            let selected_clone = Arc::clone(&selected_transaction);
            self.widgets
                .tree_view_all_transactions
                .connect_row_activated(move |tree_view, tree_path, _| {
                    if let Ok(mut tx) = selected_clone.lock() {
                        *tx = Self::get_selected_row(tree_view, tree_path);
                    }
                    b.set_sensitive(true);
                });
            let window_clone = widgets.window_blocks.clone();
            let request_button = widgets.request_button.clone();
            button.connect_clicked(move |_| {
                request_button.set_sensitive(false);
                widgets.window_blocks.show_all();
            });
            let request_button = widgets.request_button.clone();
            let selected_block = Arc::new(Mutex::new(Vec::new()));
            let selected_clone = Arc::clone(&selected_block);
            widgets
                .tree_view_blocks
                .connect_row_activated(move |tree_view, tree_path, _| {
                    if let Ok(mut block) = selected_clone.lock() {
                        *block = Self::get_selected_row(tree_view, tree_path);
                    }
                    request_button.set_sensitive(true);
                });
            widgets.request_button.connect_clicked(move |_| {
                let selected_block = Arc::clone(&selected_block);
                let selected_transaction = Arc::clone(&selected_transaction);
                if let Ok(tx) = selected_transaction.lock() {
                    if let Ok(block) = selected_block.lock() {
                        let message = InterfaceMessages::InclusionProof(
                            (*block.clone()).to_vec(),
                            (*tx.clone()).to_vec(),
                        );
                        if sender_proof_order.send(message).is_ok() {}
                    }
                }
                window_clone.hide();
                button.set_sensitive(false);
            });
        }
    }

    /// Adds the received headers to the header list in debugging.
    fn set_data_headers(headers: BlockHeader, widgets: &WidgetsGtk) {
        let values = headers.get_csv_format();
        let row = widgets.list_headers.append();
        for (i, value) in values.iter().enumerate().take(HEADER_COLUMNS) {
            widgets
                .list_headers
                .set_value(&row, i as u32, &value.to_value());
        }
    }

    /// Adds the received blocks to the block list to request proof of inclusion.
    fn set_data_blocks(block: BlockMessage, widgets: &WidgetsGtk, block_id: i32, total_blocks: usize) {
        let mut values = Vec::new();
        values.push(block_id.to_string());
        values.push(block.tx_count.number.to_string());
        let mut prev_block_header_hash_to_str = String::new();
        for byte in block.block_header.previous_block_header_hash.iter() {
            prev_block_header_hash_to_str.push_str(&format!("{:02X}", byte));
        }
        values.push(prev_block_header_hash_to_str);
        let mut merkle_root_hash_to_str = String::new();
        for byte in block.block_header.merkle_root_hash.iter() {
            merkle_root_hash_to_str.push_str(&format!("{:02X}", byte));
        }
        values.push(merkle_root_hash_to_str);
        let row = widgets.list_blocks.append();
        for (i, value) in values.iter().enumerate().take(BLOCKS_COLUMNS) {
            widgets
                .list_blocks
                .set_value(&row, i as u32, &value.to_value());
        }
        let current_percentage = widgets.progress_bar.get_fraction();
        let another_block = 1.0 / total_blocks as f64;
        widgets.progress_bar.set_fraction(current_percentage + another_block);
    }

    fn show_transactions(transactions: (bool, TXMessage, bool), widgets: &WidgetsGtk) {
        let values = Self::modify_to_show_transaction(transactions);
        let row = widgets.list_all_transactions.append();
        for (i, value) in values.iter().enumerate().take(TRANSACTIONS_COLUMNS) {
            widgets
                .list_all_transactions
                .set_value(&row, i as u32, &value.to_value());
        }
    }

    fn show_my_created_transactions(transactions: TXMessage, widgets: &WidgetsGtk) {
        let values = Self::modify_to_show_my_transaction(transactions);
        let row = widgets.list_my_transactions.append();
        for (i, value) in values
            .iter()
            .enumerate()
            .take(MY_CREATED_TRANSACTIONS_COLUMNS)
        {
            widgets
                .list_my_transactions
                .set_value(&row, i as u32, &value.to_value());
        }
    }

    fn show_popup_inclusion(widgets: &WidgetsGtk, show: bool) {
        let message = if show {
            "The transaction is included in the block."
        } else {
            "The transaction is not included in the block."
        };
        widgets.popup.set_markup(message);
        widgets.popup.run();
        widgets.popup.hide();
    }

    /// Receives the names of wallets that have been authorized to be created by the node and adds them to the current wallet list.
    fn set_wallet_data_names(builder: &Builder, wallet_name: String) {
        let wallet: Option<ComboBoxText> = builder.get_object("wallet_switch");
        if let Some(view) = wallet {
            view.append_text(wallet_name.as_str());
        }
    }

    /// Receives a hash from the node with all the labels to be modified with new data upon a wallet change event.
    fn actual_data(builder: &Builder, actual_wallet_data: HashMap<String, String>) {
        let builder = builder.clone();
        for (key, value) in actual_wallet_data.iter() {
            Self::set_label(&builder, key, value);
        }
    }

    /// Match and loop -----------------------

    fn receive_message_from_node(
        &mut self,
        sender_to_node: Sender<InterfaceMessages>,
        receiver_from_node: Receiver<InterfaceMessages>,
    ) {
        self.send_button(sender_to_node.clone());
        self.add_wallet_button(sender_to_node.clone());
        self.wallet_switch_button(sender_to_node.clone());
        self.proof_of_inclusion_button(sender_to_node);
        let widgets = self.widgets.clone();
        let builder = self.builder.clone();
        receiver_from_node.attach(None, move |message| {
            let widgets = widgets.clone();
            let builder = builder.clone();
            Self::handle_messages(builder, message, widgets);
            glib::Continue(true)
        });
    }

    /// Matches all communication channels with the node, and depending on which one is triggered, sends the necessary command to be executed.
    fn handle_messages(builder: Builder, message: InterfaceMessages, widgets: WidgetsGtk) {
        match message {
            InterfaceMessages::DebugHeaders(headers) => {
                Self::set_data_headers(headers, &widgets);
            }
            InterfaceMessages::DebugBlocks(id, blocks, total_blocks) => {
                Self::set_data_blocks(blocks, &widgets, id, total_blocks);
            }
            InterfaceMessages::AllTransactions(confirmed, transactions, belongs) => {
                Self::show_transactions((confirmed, transactions, belongs), &widgets);
            }
            InterfaceMessages::MyTransactions(transactions) => {
                Self::show_my_created_transactions(transactions, &widgets);
            }
            InterfaceMessages::WalletName(data) => {
                Self::set_wallet_data_names(&builder, data);
            }
            InterfaceMessages::ActualWallet(actual_wallet_data) => {
                Self::actual_data(&builder, actual_wallet_data);
            }
            InterfaceMessages::InclusionProofResult(show_popup) => {
                Self::show_popup_inclusion(&widgets, show_popup);
            }
            InterfaceMessages::Open(open_screen) => {
                Self::open_signal(&builder, open_screen);
            }
            _ => {}
        }
    }

    /// Main function where I display the necessary information on the screen and execute the functions.
    pub fn show_window(
        &mut self,
        sender_to_node: Sender<InterfaceMessages>,
        receiver_from_node: Receiver<InterfaceMessages>,
    ) {
        self.receive_message_from_node(sender_to_node.clone(), receiver_from_node);

        let window: Option<Window> = self.builder.get_object("window");

        if let Some(w) = window {
            self.close_button(&w, sender_to_node);
            w.show_all();
        };
        gtk::main();
    }
}
