# Plan for RithmicClient Full API Coverage

## Objective
Implement all missing client-side methods in `src/client.rs` to achieve 100% coverage of the Rithmic API sender functionalities. Each new method will call the appropriate `RithmicSenderApi` request, send it via the corresponding "plant" worker, and process its response(s).

## Phase 1: Implement Missing General/Connection Client Methods
- [ ] Implement `request_logout`
- [ ] Implement `request_heartbeat`

## Phase 2: Implement Missing Market Data Client Methods
- [ ] Implement `get_instrument_by_underlying` (for `request_get_instrument_by_underlying`)
- [ ] Implement `subscribe_market_data_by_underlying` (for `request_market_data_update_by_underlying`)
- [ ] Implement `get_tick_size_type_table` (for `request_give_tick_size_type_table`)
- [ ] Implement `get_product_codes` (for `request_product_codes`)
- [ ] Implement `get_depth_by_order_snapshot` (for `request_depth_by_order_snapshot`)
- [ ] Implement `subscribe_depth_by_order_updates` (for `request_depth_by_order_updates`)
- [ ] Implement `get_volume_at_price` (for `request_get_volume_at_price`)
- [ ] Implement `get_auxilliary_reference_data` (for `request_auxilliary_reference_data`)

## Phase 3: Implement Missing History Client Methods
- [ ] Implement `get_time_bar_updates` (for `request_time_bar_update`)
- [ ] Implement `get_tick_bar_updates` (for `request_tick_bar_update`)
- [ ] Implement `get_volume_profile_minute_bars` (for `request_volume_profile_minute_bars`)
- [ ] Implement `resume_bars` (for `request_resume_bars`)

## Phase 4: Implement Missing Order Management Client Methods
- [ ] Implement `get_login_info` (for `request_login_info`)
- [ ] Implement `get_account_rms_info` (for `request_account_rms_info`)
- [ ] Implement `get_product_rms_info` (for `request_product_rms_info`)
- [ ] Implement `get_order_history_dates` (for `request_show_order_history_dates`)
- [ ] Implement `get_order_history_summary` (for `request_show_order_history_summary`)
- [ ] Implement `get_order_history_detail` (for `request_show_order_history_detail`)
- [ ] Implement `update_target_bracket_level` (for `request_update_target_bracket_level`)
- [ ] Implement `update_stop_bracket_level` (for `request_update_stop_bracket_level`)
- [ ] Implement `subscribe_to_bracket_updates` (for `request_subscribe_to_bracket_updates`)
- [ ] Implement `show_brackets` (for `request_show_brackets`)
- [ ] Implement `show_bracket_stops` (for `request_show_bracket_stops`)
- [ ] Implement `list_exchange_permissions` (for `request_list_exchange_permissions`)
- [ ] Implement `link_orders` (for `request_link_orders`)
- [ ] Implement `get_easy_to_borrow_list` (for `request_easy_to_borrow_list`)
- [ ] Implement `modify_order_reference_data` (for `request_modify_order_reference_data`)
- [ ] Implement `get_order_session_config` (for `request_order_session_config`)
- [ ] Implement `exit_position` (for `request_exit_position`)
- [ ] Implement `replay_executions` (for `request_replay_executions`)
- [ ] Implement `subscribe_account_rms_updates` (for `request_account_rms_updates`)

## Phase 5: Implement Missing Repository Client Methods
- [ ] Implement `list_unaccepted_agreements` (for `request_list_unaccepted_agreements`)
- [ ] Implement `list_accepted_agreements` (for `request_list_accepted_agreements`)
- [ ] Implement `show_agreement` (for `request_show_agreement`)

## Implementation Strategy for each method:
1.  Add a new `pub async fn` method to the `impl RithmicClient` block in `src/client.rs`.
2.  Inside, acquire a lock on `self.sender_api` and call the corresponding `sender.request_...()` method to get the `(payload, request_id)`.
3.  Based on the expected response type (single or stream), call either `self.send_single_command_to_plant()` or `self.send_stream_command_to_plant()`, specifying the correct `plant_name` (Ticker, History, Order, or PnL).
4.  Process the returned `Result` (e.g., awaiting a `oneshot::Receiver` or iterating through an `mpsc::Receiver`) and extract the relevant `RithmicMessage` variant.
5.  Return the specific data structure or a `Result<(), Report>` if no specific data is expected back.
6.  Add appropriate tracing logs (`info!`, `debug!`, `error!`) for method calls, successes, and failures.
7.  Ensure error handling is consistent, returning `eyre::Report` for failures.

**Note:** The `RithmicResponse` receiver already provides full access to all decoded incoming messages. The goal here is to provide convenient client methods for *sending* all available requests and cleanly retrieving their primary responses. Updates and unsolicited messages will still be delivered via the main `event_rx` channel obtained from `client.connect()`.