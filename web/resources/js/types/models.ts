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
    address: string;
    balance_usdc: string;
    is_active: boolean;
    strategies_count?: number;
}

export interface WalletStrategy {
    id: number;
    is_running: boolean;
    max_position_usdc: string;
    wallet: { id: number; label: string | null; address: string };
}

export interface BacktestResult {
    id: number;
    total_trades: number | null;
    win_rate: string | null;
    total_pnl_usdc: string | null;
    max_drawdown: string | null;
    sharpe_ratio: string | null;
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
    stoploss_pct: number;
    take_profit_pct: number;
    max_position_usdc: number;
    max_trades_per_slot: number;
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
    type: 'input' | 'indicator' | 'comparator' | 'logic' | 'action';
    data: Record<string, unknown>;
    position?: { x: number; y: number };
}

export interface GraphEdge {
    source: string;
    target: string;
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
