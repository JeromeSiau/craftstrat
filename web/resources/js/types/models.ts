// Pagination wrapper from Laravel's paginate()
export interface Paginated<T> {
    data: T[];
    current_page: number;
    last_page: number;
    per_page: number;
    total: number;
    links: Array<{ url: string | null; label: string; active: boolean }>;
}

export interface Strategy {
    id: number;
    name: string;
    description: string | null;
    mode: 'form' | 'node';
    graph: FormModeGraph | NodeModeGraph;
    is_active: boolean;
    wallets_count?: number;
    wallet_strategies?: WalletStrategy[];
    backtest_results?: BacktestResult[];
    created_at: string;
}

export interface Wallet {
    id: number;
    label: string | null;
    signer_address: string;
    safe_address: string | null;
    status: 'pending' | 'deploying' | 'deployed' | 'failed';
    balance_usdc: string;
    is_active: boolean;
    deployed_at: string | null;
    strategies_count?: number;
}

export interface WalletStrategy {
    id: number;
    is_running: boolean;
    is_paper: boolean;
    max_position_usdc: string;
    markets: string[];
    wallet: { id: number; label: string | null; safe_address: string | null; signer_address: string };
}

export interface BacktestResult {
    id: number;
    total_trades: number | null;
    win_rate: string | null;
    total_pnl_usdc: string | null;
    max_drawdown: string | null;
    sharpe_ratio: string | null;
    market_filter: string[] | null;
    date_from: string | null;
    date_to: string | null;
    created_at: string;
    strategy: { id: number; name: string; graph?: Record<string, unknown> };
    result_detail?: {
        trades?: BacktestTrade[];
    } | null;
}

export interface DashboardStats {
    active_strategies: number;
    total_strategies: number;
    total_wallets: number;
    total_pnl_usdc: string;
    running_assignments: number;
}

// Strategy graph types for form mode
export interface StrategyRule {
    id: string;
    indicator: string;
    operator: string;
    value: number | [number, number];
}

export interface ConditionGroup {
    id: string;
    type: 'AND' | 'OR';
    rules: StrategyRule[];
}

export interface StrategyAction {
    signal: 'buy' | 'sell';
    outcome: 'UP' | 'DOWN';
    size_mode: 'fixed' | 'proportional';
    size_usdc: number;
    order_type: 'market' | 'limit';
}

export interface StrategyRisk {
    stoploss_pct: number | null;
    take_profit_pct: number | null;
    max_position_usdc: number;
    max_trades_per_slot: number;
    daily_loss_limit_usdc: number | null;
    cooldown_seconds: number | null;
    prevent_duplicates: boolean;
}

export interface FormModeGraph {
    mode: 'form';
    conditions: ConditionGroup[];
    action: StrategyAction;
    risk: StrategyRisk;
}

// Strategy graph types for node mode
export interface GraphNode {
    id: string;
    type: 'input' | 'indicator' | 'comparator' | 'logic' | 'action' | 'not' | 'if_else' | 'math' | 'ev_calculator' | 'kelly' | 'cancel' | 'notify' | 'api_fetch';
    data: Record<string, unknown>;
    position?: { x: number; y: number };
}

export interface GraphEdge {
    source: string;
    target: string;
    sourceHandle?: string | null;
    targetHandle?: string | null;
}

export interface NodeModeGraph {
    mode: 'node';
    nodes: GraphNode[];
    edges: GraphEdge[];
}

export interface BacktestTrade {
    tick_index: number;
    side: 'buy' | 'sell';
    outcome: 'UP' | 'DOWN';
    entry_price: number;
    exit_price: number | null;
    pnl: number;
    cumulative_pnl: number;
}

export interface Trade {
    id: number;
    market_id: string | null;
    side: string | null;
    outcome: string | null;
    price: string | null;
    size_usdc: string | null;
    status: string;
    is_paper: boolean;
    executed_at: string | null;
}

export interface LiveStatsEntry {
    total_trades: number;
    win_rate: string | null;
    total_pnl_usdc: string | null;
}

export interface LiveStats {
    live: LiveStatsEntry;
    paper: LiveStatsEntry;
}

// Slot Analytics types
export interface SlotAnalyticsSummary {
    total_slots: number;
    resolved_slots: number;
    unresolved_slots: number;
    total_snapshots: number;
    last_snapshot_at: string | null;
}

export interface HeatmapCell {
    time_bin: string;
    move_bin: string;
    total: number;
    wins: number;
    win_rate: number;
}

export interface CalibrationPoint {
    bid_bucket: number;
    avg_bid: number;
    win_rate: number;
    sample_count: number;
}

export interface SymbolStats {
    symbol: string;
    total: number;
    wins: number;
    win_rate: number;
}

export interface StoplossThreshold {
    threshold: number;
    triggered: number;
    true_saves: number;
    false_exits: number;
    precision: number;
}

export interface TimeStats {
    period: number;
    total: number;
    wins: number;
    win_rate: number;
}

export interface SlotAnalyticsData {
    summary: SlotAnalyticsSummary;
    heatmap: HeatmapCell[];
    calibration: CalibrationPoint[];
    by_symbol: SymbolStats[];
    stoploss_sweep: StoplossThreshold[];
    by_hour: TimeStats[];
    by_day: TimeStats[];
}

export interface AnalyticsFilters {
    slot_duration: number;
    symbols: string[];
    hours: number;
}
