import { Head, Link, router } from '@inertiajs/react';
import { Pencil, RefreshCw, Trash2 } from 'lucide-react';
import { useState } from 'react';
import {
    index,
    show,
    destroy,
    rerun,
} from '@/actions/App/Http/Controllers/BacktestController';
import { show as showStrategy } from '@/actions/App/Http/Controllers/StrategyController';
import { PnlChart } from '@/components/charts/pnl-chart';
import ConfirmDialog from '@/components/confirm-dialog';
import MetricCard from '@/components/metric-card';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';
import AppLayout from '@/layouts/app-layout';
import { MARKET_LABEL_MAP } from '@/lib/constants';
import {
    formatBps,
    formatPercentage,
    formatPnl,
    formatWinRate,
} from '@/lib/formatters';
import type { BreadcrumbItem } from '@/types';
import type { BacktestResult } from '@/types/models';

function formatPrice(value: number | null | undefined): string {
    if (value === null || value === undefined) return '-';

    return `$${value.toFixed(4)}`;
}

function bpsColorClass(
    value: number | null | undefined,
    positiveIsGood: boolean,
): string {
    if (value === null || value === undefined || value === 0)
        return 'text-muted-foreground';

    if (positiveIsGood) {
        return value > 0
            ? 'text-emerald-600 dark:text-emerald-400'
            : 'text-red-500 dark:text-red-400';
    }

    return value > 0
        ? 'text-red-500 dark:text-red-400'
        : 'text-emerald-600 dark:text-emerald-400';
}

export default function BacktestsShow({ result }: { result: BacktestResult }) {
    const [rerunning, setRerunning] = useState(false);

    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Backtests', href: index.url() },
        { title: `#${result.id}`, href: show.url(result.id) },
    ];

    const pnlValue = parseFloat(result.total_pnl_usdc || '0');
    const drawdownValue = parseFloat(result.max_drawdown || '0');

    const metrics = [
        { label: 'Total Trades', value: result.total_trades ?? '-' },
        { label: 'Win Rate', value: formatWinRate(result.win_rate) },
        {
            label: 'PnL',
            value: formatPnl(result.total_pnl_usdc),
            trend: (pnlValue > 0 ? 'up' : pnlValue < 0 ? 'down' : 'neutral') as
                | 'up'
                | 'down'
                | 'neutral',
        },
        {
            label: 'Max Drawdown',
            value: formatPercentage(result.max_drawdown),
            trend: (drawdownValue < 0 ? 'down' : 'neutral') as
                | 'up'
                | 'down'
                | 'neutral',
        },
        { label: 'Sharpe Ratio', value: result.sharpe_ratio ?? '-' },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={`Backtest #${result.id}`} />
            <div className="p-4 md:p-8">
                <div className="mb-8 flex items-start justify-between gap-4">
                    <div>
                        <h1 className="text-2xl font-bold tracking-tight">
                            Backtest #{result.id}
                        </h1>
                        <p className="mt-1 text-muted-foreground">
                            Strategy: {result.strategy.name}
                            {result.date_from && result.date_to && (
                                <>
                                    {' '}
                                    ·{' '}
                                    {new Date(
                                        result.date_from,
                                    ).toLocaleDateString()}{' '}
                                    –{' '}
                                    {new Date(
                                        result.date_to,
                                    ).toLocaleDateString()}
                                </>
                            )}
                        </p>
                        <div className="mt-2 flex flex-wrap gap-1">
                            {result.market_filter?.length ? (
                                result.market_filter.map((m) => (
                                    <span
                                        key={m}
                                        className="rounded-md bg-muted px-1.5 py-0.5 text-xs text-muted-foreground"
                                    >
                                        {MARKET_LABEL_MAP[m] ?? m}
                                    </span>
                                ))
                            ) : (
                                <span className="rounded-md bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                                    All markets
                                </span>
                            )}
                        </div>
                    </div>
                    <div className="flex shrink-0 gap-2">
                        <Button variant="outline" size="sm" asChild>
                            <Link href={showStrategy.url(result.strategy.id)}>
                                <Pencil className="size-3.5" />
                                Edit Strategy
                            </Link>
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            disabled={rerunning}
                            onClick={() => {
                                setRerunning(true);
                                router.post(
                                    rerun.url(result.id),
                                    {},
                                    {
                                        onFinish: () => setRerunning(false),
                                    },
                                );
                            }}
                        >
                            <RefreshCw
                                className={`size-3.5 ${rerunning ? 'animate-spin' : ''}`}
                            />
                            {rerunning ? 'Running...' : 'Rerun'}
                        </Button>
                        <ConfirmDialog
                            trigger={
                                <Button variant="destructive" size="sm">
                                    <Trash2 className="size-3.5" />
                                    Delete
                                </Button>
                            }
                            title="Delete Backtest"
                            description="Are you sure you want to delete this backtest result? This action cannot be undone."
                            confirmLabel="Delete"
                            onConfirm={() =>
                                router.delete(destroy.url(result.id))
                            }
                        />
                    </div>
                </div>

                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    {metrics.map((m) => (
                        <MetricCard
                            key={m.label}
                            label={m.label}
                            value={m.value}
                            trend={m.trend}
                        />
                    ))}
                </div>

                {result.result_detail?.trades &&
                    result.result_detail.trades.length > 0 && (
                        <>
                            <Card className="mt-6 border-l-4 border-l-emerald-500/50">
                                <CardHeader>
                                    <CardTitle>Cumulative PnL</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <PnlChart
                                        trades={result.result_detail.trades}
                                    />
                                </CardContent>
                            </Card>

                            <Card className="mt-6 border-l-4 border-l-cyan-500/50">
                                <CardHeader>
                                    <CardTitle>Execution Detail</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>#</TableHead>
                                                <TableHead>Market</TableHead>
                                                <TableHead>Entry</TableHead>
                                                <TableHead>
                                                    Entry Slip
                                                </TableHead>
                                                <TableHead>
                                                    Entry Depth
                                                </TableHead>
                                                <TableHead>Exit</TableHead>
                                                <TableHead>Exit Slip</TableHead>
                                                <TableHead>PnL</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {result.result_detail.trades.map(
                                                (trade) => (
                                                    <TableRow
                                                        key={`${trade.tick_index}-${trade.symbol ?? 'trade'}`}
                                                    >
                                                        <TableCell className="text-muted-foreground">
                                                            {trade.tick_index +
                                                                1}
                                                        </TableCell>
                                                        <TableCell className="max-w-[220px] truncate font-mono text-xs">
                                                            {trade.symbol ??
                                                                '-'}
                                                        </TableCell>
                                                        <TableCell className="tabular-nums">
                                                            <div>
                                                                {formatPrice(
                                                                    trade.entry_price,
                                                                )}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">
                                                                ref{' '}
                                                                {formatPrice(
                                                                    trade.entry_reference_price,
                                                                )}
                                                            </div>
                                                        </TableCell>
                                                        <TableCell
                                                            className={`tabular-nums ${bpsColorClass(trade.entry_slippage_bps, false)}`}
                                                        >
                                                            {formatBps(
                                                                String(
                                                                    trade.entry_slippage_bps,
                                                                ),
                                                            )}
                                                        </TableCell>
                                                        <TableCell className="text-muted-foreground tabular-nums">
                                                            {(
                                                                trade.entry_depth_ratio *
                                                                100
                                                            ).toFixed(1)}
                                                            %
                                                        </TableCell>
                                                        <TableCell className="tabular-nums">
                                                            <div>
                                                                {formatPrice(
                                                                    trade.exit_price,
                                                                )}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">
                                                                ref{' '}
                                                                {formatPrice(
                                                                    trade.exit_reference_price,
                                                                )}
                                                            </div>
                                                        </TableCell>
                                                        <TableCell
                                                            className={`tabular-nums ${bpsColorClass(trade.exit_slippage_bps, false)}`}
                                                        >
                                                            {trade.exit_slippage_bps ===
                                                                null ||
                                                            trade.exit_slippage_bps ===
                                                                undefined
                                                                ? '-'
                                                                : formatBps(
                                                                      String(
                                                                          trade.exit_slippage_bps,
                                                                      ),
                                                                  )}
                                                        </TableCell>
                                                        <TableCell
                                                            className={`tabular-nums ${trade.pnl >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-500 dark:text-red-400'}`}
                                                        >
                                                            {trade.pnl >= 0
                                                                ? '+'
                                                                : ''}
                                                            {trade.pnl.toFixed(
                                                                2,
                                                            )}
                                                        </TableCell>
                                                    </TableRow>
                                                ),
                                            )}
                                        </TableBody>
                                    </Table>
                                </CardContent>
                            </Card>
                        </>
                    )}
            </div>
        </AppLayout>
    );
}
