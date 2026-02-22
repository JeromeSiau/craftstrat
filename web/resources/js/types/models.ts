export interface Strategy {
    id: number;
    name: string;
    description: string | null;
    mode: string;
    graph: Record<string, unknown>;
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
    indicator: string;
    operator: string;
    value: number | [number, number];
}

export interface ConditionGroup {
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
