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
