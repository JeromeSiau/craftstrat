import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';
import type { WalletStrategy, BacktestResult, FormModeGraph, ConditionGroup, StrategyRule } from '@/types/models';
import { indicators } from '@/components/strategy/indicator-options';
import { index, show, activate, deactivate, destroy } from '@/actions/App/Http/Controllers/StrategyController';

interface StrategyShowProps {
    id: number;
    name: string;
    description: string | null;
    mode: string;
    graph: Record<string, unknown>;
    is_active: boolean;
    wallet_strategies: WalletStrategy[];
    backtest_results: BacktestResult[];
}

const indicatorLabelMap = Object.fromEntries(
    indicators.map((i) => [i.value, i.label]),
);

function formatRuleValue(rule: StrategyRule): string {
    if (rule.operator === 'between' && Array.isArray(rule.value)) {
        return `${rule.value[0]} and ${rule.value[1]}`;
    }
    return String(rule.value);
}

function isFormModeGraph(graph: Record<string, unknown>): graph is FormModeGraph {
    return graph.mode === 'form' && Array.isArray(graph.conditions);
}

function StrategyRulesDisplay({ graph }: { graph: FormModeGraph }) {
    return (
        <div className="mt-4 space-y-4">
            <h3 className="text-sm font-semibold">Strategy Rules</h3>

            {graph.conditions.map((group: ConditionGroup, groupIndex: number) => (
                <div key={groupIndex} className="rounded-md border p-3">
                    <p className="mb-2 text-xs font-medium text-muted-foreground">
                        Condition Group {groupIndex + 1} ({group.type})
                    </p>
                    <ul className="space-y-1 text-sm">
                        {group.rules.map((rule: StrategyRule, ruleIndex: number) => (
                            <li key={ruleIndex} className="flex items-center gap-1.5">
                                <span className="font-medium">
                                    {indicatorLabelMap[rule.indicator] || rule.indicator}
                                </span>
                                <span className="text-muted-foreground">{rule.operator}</span>
                                <span>{formatRuleValue(rule)}</span>
                            </li>
                        ))}
                    </ul>
                </div>
            ))}

            <div className="rounded-md border p-3">
                <p className="mb-2 text-xs font-medium text-muted-foreground">Action</p>
                <dl className="grid grid-cols-2 gap-1 text-sm">
                    <dt className="text-muted-foreground">Signal</dt>
                    <dd className="capitalize">{graph.action.signal}</dd>
                    <dt className="text-muted-foreground">Outcome</dt>
                    <dd>{graph.action.outcome}</dd>
                    <dt className="text-muted-foreground">Size</dt>
                    <dd>{graph.action.size_usdc} USDC</dd>
                    <dt className="text-muted-foreground">Order Type</dt>
                    <dd className="capitalize">{graph.action.order_type}</dd>
                </dl>
            </div>

            <div className="rounded-md border p-3">
                <p className="mb-2 text-xs font-medium text-muted-foreground">Risk</p>
                <dl className="grid grid-cols-2 gap-1 text-sm">
                    <dt className="text-muted-foreground">Stop Loss</dt>
                    <dd>{graph.risk.stoploss_pct}%</dd>
                    <dt className="text-muted-foreground">Take Profit</dt>
                    <dd>{graph.risk.take_profit_pct}%</dd>
                    <dt className="text-muted-foreground">Max Position</dt>
                    <dd>{graph.risk.max_position_usdc} USDC</dd>
                    <dt className="text-muted-foreground">Max Trades / Slot</dt>
                    <dd>{graph.risk.max_trades_per_slot}</dd>
                </dl>
            </div>
        </div>
    );
}

export default function StrategiesShow({ strategy }: { strategy: StrategyShowProps }) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: index.url() },
        { title: strategy.name, href: show.url(strategy.id) },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={strategy.name} />
            <div className="p-6">
                <div className="mb-6 flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold">{strategy.name}</h1>
                        {strategy.description && (
                            <p className="mt-1 text-muted-foreground">
                                {strategy.description}
                            </p>
                        )}
                    </div>
                    <div className="flex gap-2">
                        {strategy.is_active ? (
                            <Button
                                variant="outline"
                                onClick={() =>
                                    router.post(
                                        deactivate.url(strategy.id),
                                    )
                                }
                            >
                                Deactivate
                            </Button>
                        ) : (
                            <Button
                                onClick={() =>
                                    router.post(
                                        activate.url(strategy.id),
                                    )
                                }
                            >
                                Activate
                            </Button>
                        )}
                        <Button
                            variant="destructive"
                            onClick={() => {
                                if (confirm('Are you sure you want to delete this strategy? This action cannot be undone.')) {
                                    router.delete(destroy.url(strategy.id));
                                }
                            }}
                        >
                            Delete
                        </Button>
                    </div>
                </div>

                <div className="grid gap-6 md:grid-cols-2">
                    <div className="rounded-lg border border-sidebar-border p-4">
                        <h2 className="mb-3 font-semibold">Configuration</h2>
                        <dl className="space-y-2 text-sm">
                            <div className="flex justify-between">
                                <dt className="text-muted-foreground">Mode</dt>
                                <dd>{strategy.mode}</dd>
                            </div>
                            <div className="flex justify-between">
                                <dt className="text-muted-foreground">
                                    Status
                                </dt>
                                <dd>
                                    {strategy.is_active ? 'Active' : 'Inactive'}
                                </dd>
                            </div>
                        </dl>

                        {strategy.graph && isFormModeGraph(strategy.graph) && (
                            <StrategyRulesDisplay graph={strategy.graph} />
                        )}
                    </div>

                    <div className="rounded-lg border border-sidebar-border p-4">
                        <h2 className="mb-3 font-semibold">Assigned Wallets</h2>
                        {strategy.wallet_strategies.length === 0 ? (
                            <p className="text-sm text-muted-foreground">
                                No wallets assigned.
                            </p>
                        ) : (
                            <ul className="space-y-2 text-sm">
                                {strategy.wallet_strategies.map((ws) => (
                                    <li
                                        key={ws.id}
                                        className="flex items-center justify-between"
                                    >
                                        <span className="font-mono text-xs">
                                            {ws.wallet.label ||
                                                ws.wallet.address.slice(0, 10) +
                                                    '...'}
                                        </span>
                                        <span
                                            className={
                                                ws.is_running
                                                    ? 'text-green-600'
                                                    : 'text-gray-400'
                                            }
                                        >
                                            {ws.is_running
                                                ? 'Running'
                                                : 'Stopped'}
                                        </span>
                                    </li>
                                ))}
                            </ul>
                        )}
                    </div>
                </div>

                <div className="mt-6 rounded-lg border border-sidebar-border p-4">
                    <h2 className="mb-3 font-semibold">Recent Backtests</h2>
                    {strategy.backtest_results.length === 0 ? (
                        <p className="text-sm text-muted-foreground">
                            No backtests yet.
                        </p>
                    ) : (
                        <div className="overflow-x-auto">
                            <table className="w-full text-sm">
                                <thead>
                                    <tr className="border-b text-left text-muted-foreground">
                                        <th className="pb-2">Date</th>
                                        <th className="pb-2">Trades</th>
                                        <th className="pb-2">Win Rate</th>
                                        <th className="pb-2">PnL</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {strategy.backtest_results.map((bt) => (
                                        <tr key={bt.id} className="border-b">
                                            <td className="py-2">
                                                {new Date(
                                                    bt.created_at,
                                                ).toLocaleDateString()}
                                            </td>
                                            <td className="py-2">
                                                {bt.total_trades}
                                            </td>
                                            <td className="py-2">
                                                {bt.win_rate
                                                    ? `${(parseFloat(bt.win_rate) * 100).toFixed(1)}%`
                                                    : '-'}
                                            </td>
                                            <td
                                                className={`py-2 ${bt.total_pnl_usdc && parseFloat(bt.total_pnl_usdc) >= 0 ? 'text-green-600' : 'text-red-600'}`}
                                            >
                                                {bt.total_pnl_usdc
                                                    ? `$${parseFloat(bt.total_pnl_usdc).toFixed(2)}`
                                                    : '-'}
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    )}
                </div>
            </div>
        </AppLayout>
    );
}
