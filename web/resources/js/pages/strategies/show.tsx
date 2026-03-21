import { Deferred, Head, Link, router, useForm } from '@inertiajs/react';
import {
    Activity,
    ArrowLeftRight,
    CircleHelp,
    FlaskConical,
    LineChart,
    OctagonX,
    Pencil,
    Settings2,
    TrendingUp,
    Wallet,
    X,
} from 'lucide-react';
import { run as runBacktest } from '@/actions/App/Http/Controllers/BacktestController';
import {
    index,
    show,
    edit,
    activate,
    deactivate,
    destroy,
    kill,
    unkill,
} from '@/actions/App/Http/Controllers/StrategyController';
import BacktestResultsTable from '@/components/backtest-results-table';
import ConfirmDialog from '@/components/confirm-dialog';
import InputError from '@/components/input-error';
import MetricCard from '@/components/metric-card';
import StatusBadge from '@/components/status-badge';
import StrategyRulesDisplay, {
    isFormModeGraph,
} from '@/components/strategy/strategy-rules-display';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import AppLayout from '@/layouts/app-layout';
import { MARKET_OPTIONS, MARKET_LABEL_MAP } from '@/lib/constants';
import { formatBps, formatPnl, formatWinRate } from '@/lib/formatters';
import { removeStrategy } from '@/routes/wallets';
import type { BreadcrumbItem } from '@/types';
import type { LiveStats, Strategy, Trade } from '@/types/models';

function LiveDataSkeleton() {
    return (
        <>
            <div className="mt-6 grid gap-4 sm:grid-cols-2 xl:grid-cols-5">
                {Array.from({ length: 5 }).map((_, i) => (
                    <Card key={i} className="relative overflow-hidden">
                        <CardContent className="pt-5 pb-5">
                            <div className="space-y-2">
                                <Skeleton className="h-3 w-24" />
                                <Skeleton className="h-8 w-20" />
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
            <Card className="mt-6">
                <CardHeader>
                    <Skeleton className="h-5 w-32" />
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        {Array.from({ length: 5 }).map((_, i) => (
                            <Skeleton key={i} className="h-4 w-full" />
                        ))}
                    </div>
                </CardContent>
            </Card>
        </>
    );
}

interface Props {
    strategy: Strategy;
    liveStats?: LiveStats;
    recentTrades?: Trade[];
}

const markoutTooltipText =
    'Side-adjusted post-fill drift measured against the first mid price at least 60 seconds after execution. Positive means the market moved in your favor after the fill.';

function formatPrice(value: string | null): string {
    if (!value) return '-';

    const parsed = parseFloat(value);
    if (Number.isNaN(parsed)) return '-';

    return `$${parsed.toFixed(4)}`;
}

function bpsColorClass(value: string | null, positiveIsGood: boolean): string {
    if (!value) return 'text-muted-foreground';

    const parsed = parseFloat(value);
    if (Number.isNaN(parsed) || parsed === 0) return 'text-muted-foreground';

    if (positiveIsGood) {
        return parsed > 0
            ? 'text-emerald-600 dark:text-emerald-400'
            : 'text-red-500 dark:text-red-400';
    }

    return parsed > 0
        ? 'text-red-500 dark:text-red-400'
        : 'text-emerald-600 dark:text-emerald-400';
}

function LabelWithTooltip({
    label,
    tooltip,
}: {
    label: string;
    tooltip: string;
}) {
    return (
        <span className="inline-flex items-center gap-1">
            <span>{label}</span>
            <Tooltip>
                <TooltipTrigger asChild>
                    <button
                        type="button"
                        className="rounded-sm text-muted-foreground/70 outline-none transition-colors hover:text-foreground"
                        aria-label={`${label} help`}
                    >
                        <CircleHelp className="size-3.5" />
                    </button>
                </TooltipTrigger>
                <TooltipContent className="max-w-xs leading-relaxed">
                    {tooltip}
                </TooltipContent>
            </Tooltip>
        </span>
    );
}

export default function StrategiesShow({
    strategy,
    liveStats,
    recentTrades,
}: Props) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: index.url() },
        { title: strategy.name, href: show.url(strategy.id) },
    ];

    const backtestForm = useForm({
        date_from: '',
        date_to: '',
        market_filter: [] as string[],
    });

    function handleBacktestSubmit(e: React.FormEvent): void {
        e.preventDefault();
        backtestForm.post(runBacktest.url(strategy.id));
    }

    const currentStats = liveStats?.live;
    const paperStats = liveStats?.paper;
    const pnlValue = currentStats?.total_pnl_usdc
        ? parseFloat(currentStats.total_pnl_usdc)
        : 0;
    const pnlTrend =
        pnlValue > 0 ? 'up' : pnlValue < 0 ? 'down' : ('neutral' as const);
    const paperPnlValue = paperStats?.total_pnl_usdc
        ? parseFloat(paperStats.total_pnl_usdc)
        : 0;
    const paperPnlTrend =
        paperPnlValue > 0
            ? 'up'
            : paperPnlValue < 0
              ? 'down'
              : ('neutral' as const);

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={strategy.name} />
            <div className="p-4 md:p-8">
                <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div>
                        <div className="flex items-center gap-3">
                            <h1 className="text-2xl font-bold tracking-tight">
                                {strategy.name}
                            </h1>
                            <StatusBadge active={strategy.is_active} />
                        </div>
                        {strategy.description && (
                            <p className="mt-1 text-muted-foreground">
                                {strategy.description}
                            </p>
                        )}
                    </div>
                    <div className="flex gap-2">
                        <Button variant="outline" asChild>
                            <Link href={edit.url(strategy.id)}>
                                <Pencil className="mr-1.5 size-4" />
                                Edit
                            </Link>
                        </Button>
                        {strategy.is_active && (
                            <>
                                <ConfirmDialog
                                    trigger={
                                        <Button
                                            variant="outline"
                                            className="border-red-500/50 text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950"
                                        >
                                            <OctagonX className="mr-1.5 size-4" />
                                            Kill
                                        </Button>
                                    }
                                    title="Activate Kill Switch"
                                    description="This will immediately stop all evaluation for this strategy across all wallets. No new trades will be placed. Use Resume to restart."
                                    confirmLabel="Kill"
                                    onConfirm={() =>
                                        router.post(kill.url(strategy.id))
                                    }
                                />
                                <Button
                                    variant="outline"
                                    onClick={() =>
                                        router.post(unkill.url(strategy.id))
                                    }
                                >
                                    Resume
                                </Button>
                            </>
                        )}
                        {strategy.is_active ? (
                            <Button
                                variant="outline"
                                onClick={() =>
                                    router.post(deactivate.url(strategy.id))
                                }
                            >
                                Deactivate
                            </Button>
                        ) : (
                            <Button
                                onClick={() =>
                                    router.post(activate.url(strategy.id))
                                }
                            >
                                Activate
                            </Button>
                        )}
                        <ConfirmDialog
                            trigger={
                                <Button variant="destructive">Delete</Button>
                            }
                            title="Delete Strategy"
                            description="Are you sure you want to delete this strategy? This action cannot be undone."
                            confirmLabel="Delete"
                            onConfirm={() =>
                                router.delete(destroy.url(strategy.id))
                            }
                        />
                    </div>
                </div>

                <div className="grid gap-6 lg:grid-cols-5">
                    <Card className="border-l-4 border-l-blue-500/50 lg:col-span-3">
                        <CardHeader>
                            <div className="flex items-center gap-3">
                                <div className="rounded-lg bg-blue-500/10 p-2 dark:bg-blue-500/15">
                                    <Settings2 className="size-4 text-blue-600 dark:text-blue-400" />
                                </div>
                                <CardTitle>Configuration</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent>
                            <dl className="grid gap-4 sm:grid-cols-2">
                                <div className="rounded-lg bg-muted/50 p-3">
                                    <dt className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                                        Mode
                                    </dt>
                                    <dd className="mt-1 font-semibold capitalize">
                                        {strategy.mode}
                                    </dd>
                                </div>
                                <div className="rounded-lg bg-muted/50 p-3">
                                    <dt className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                                        Status
                                    </dt>
                                    <dd className="mt-1 font-semibold">
                                        {strategy.is_active
                                            ? 'Active'
                                            : 'Inactive'}
                                    </dd>
                                </div>
                            </dl>

                            {strategy.graph &&
                                isFormModeGraph(strategy.graph) && (
                                    <div className="mt-6">
                                        <StrategyRulesDisplay
                                            graph={strategy.graph}
                                        />
                                    </div>
                                )}
                        </CardContent>
                    </Card>

                    <Card className="border-l-4 border-l-violet-500/50 lg:col-span-2">
                        <CardHeader>
                            <div className="flex items-center gap-3">
                                <div className="rounded-lg bg-violet-500/10 p-2 dark:bg-violet-500/15">
                                    <Wallet className="size-4 text-violet-600 dark:text-violet-400" />
                                </div>
                                <CardTitle>Assigned Wallets</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent>
                            {!strategy.wallet_strategies?.length ? (
                                <div className="py-6 text-center">
                                    <p className="text-sm text-muted-foreground">
                                        No wallets assigned yet.
                                    </p>
                                </div>
                            ) : (
                                <div className="divide-y">
                                    {strategy.wallet_strategies.map((ws) => (
                                        <div
                                            key={ws.id}
                                            className="py-3 first:pt-0 last:pb-0"
                                        >
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-2 truncate">
                                                    <span className="truncate font-mono text-sm">
                                                        {ws.wallet.label ||
                                                            `${(ws.wallet.safe_address ?? ws.wallet.signer_address).slice(0, 10)}...`}
                                                    </span>
                                                    {ws.is_paper && (
                                                        <Badge
                                                            variant="outline"
                                                            className="shrink-0 border-amber-500/50 text-amber-600 dark:text-amber-400"
                                                        >
                                                            Paper
                                                        </Badge>
                                                    )}
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    <span
                                                        className={`shrink-0 text-xs font-semibold ${
                                                            ws.is_running
                                                                ? 'text-emerald-600 dark:text-emerald-400'
                                                                : 'text-muted-foreground'
                                                        }`}
                                                    >
                                                        {ws.is_running
                                                            ? 'Running'
                                                            : 'Stopped'}
                                                    </span>
                                                    <ConfirmDialog
                                                        trigger={
                                                            <button
                                                                type="button"
                                                                className="rounded p-0.5 text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
                                                            >
                                                                <X className="size-3.5" />
                                                            </button>
                                                        }
                                                        title="Remove from Wallet"
                                                        description={`Remove this strategy from wallet "${ws.wallet.label || 'this wallet'}"? The strategy will stop running on this wallet.`}
                                                        confirmLabel="Remove"
                                                        onConfirm={() =>
                                                            router.delete(
                                                                removeStrategy.url(
                                                                    {
                                                                        wallet: ws
                                                                            .wallet
                                                                            .id,
                                                                        strategy:
                                                                            strategy.id,
                                                                    },
                                                                ),
                                                            )
                                                        }
                                                    />
                                                </div>
                                            </div>
                                            {ws.markets?.length > 0 && (
                                                <div className="mt-1.5 flex flex-wrap gap-1">
                                                    {ws.markets.map((m) => (
                                                        <span
                                                            key={m}
                                                            className="rounded-md bg-muted px-1.5 py-0.5 text-xs text-muted-foreground"
                                                        >
                                                            {MARKET_LABEL_MAP[
                                                                m
                                                            ] ?? m}
                                                        </span>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
                                    ))}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                <Deferred
                    data={['liveStats', 'recentTrades']}
                    fallback={<LiveDataSkeleton />}
                >
                    <Tabs defaultValue="live" className="mt-6">
                        <TabsList>
                            <TabsTrigger value="live">Live</TabsTrigger>
                            <TabsTrigger value="paper">Paper</TabsTrigger>
                        </TabsList>
                        <TabsContent value="live">
                            <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-5">
                                <MetricCard
                                    label="Total Trades"
                                    value={currentStats?.total_trades ?? 0}
                                    icon={Activity}
                                />
                                <MetricCard
                                    label="Win Rate"
                                    value={formatWinRate(
                                        currentStats?.win_rate ?? null,
                                    )}
                                    icon={TrendingUp}
                                />
                                <MetricCard
                                    label="PnL"
                                    value={formatPnl(
                                        currentStats?.total_pnl_usdc ?? null,
                                    )}
                                    icon={ArrowLeftRight}
                                    trend={
                                        currentStats?.total_pnl_usdc
                                            ? pnlTrend
                                            : undefined
                                    }
                                />
                                <MetricCard
                                    label="Avg Slippage"
                                    value={formatBps(
                                        currentStats?.avg_fill_slippage_bps ??
                                            null,
                                    )}
                                    icon={Settings2}
                                    trend={
                                        currentStats?.avg_fill_slippage_bps
                                            ? parseFloat(
                                                  currentStats.avg_fill_slippage_bps,
                                              ) > 0
                                                ? 'down'
                                                : parseFloat(
                                                        currentStats.avg_fill_slippage_bps,
                                                    ) < 0
                                                  ? 'up'
                                                  : 'neutral'
                                            : undefined
                                    }
                                />
                                <MetricCard
                                    label={
                                        <LabelWithTooltip
                                            label="1m Markout"
                                            tooltip={markoutTooltipText}
                                        />
                                    }
                                    value={formatBps(
                                        currentStats?.avg_markout_bps_60s ??
                                            null,
                                    )}
                                    icon={LineChart}
                                    trend={
                                        currentStats?.avg_markout_bps_60s
                                            ? parseFloat(
                                                  currentStats.avg_markout_bps_60s,
                                              ) > 0
                                                ? 'up'
                                                : parseFloat(
                                                        currentStats.avg_markout_bps_60s,
                                                    ) < 0
                                                  ? 'down'
                                                  : 'neutral'
                                            : undefined
                                    }
                                />
                            </div>
                        </TabsContent>
                        <TabsContent value="paper">
                            <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-5">
                                <MetricCard
                                    label="Total Trades"
                                    value={paperStats?.total_trades ?? 0}
                                    icon={Activity}
                                />
                                <MetricCard
                                    label="Win Rate"
                                    value={formatWinRate(
                                        paperStats?.win_rate ?? null,
                                    )}
                                    icon={TrendingUp}
                                />
                                <MetricCard
                                    label="PnL"
                                    value={formatPnl(
                                        paperStats?.total_pnl_usdc ?? null,
                                    )}
                                    icon={ArrowLeftRight}
                                    trend={
                                        paperStats?.total_pnl_usdc
                                            ? paperPnlTrend
                                            : undefined
                                    }
                                />
                                <MetricCard
                                    label="Avg Slippage"
                                    value={formatBps(
                                        paperStats?.avg_fill_slippage_bps ??
                                            null,
                                    )}
                                    icon={Settings2}
                                    trend={
                                        paperStats?.avg_fill_slippage_bps
                                            ? parseFloat(
                                                  paperStats.avg_fill_slippage_bps,
                                              ) > 0
                                                ? 'down'
                                                : parseFloat(
                                                        paperStats.avg_fill_slippage_bps,
                                                    ) < 0
                                                  ? 'up'
                                                  : 'neutral'
                                            : undefined
                                    }
                                />
                                <MetricCard
                                    label={
                                        <LabelWithTooltip
                                            label="1m Markout"
                                            tooltip={markoutTooltipText}
                                        />
                                    }
                                    value={formatBps(
                                        paperStats?.avg_markout_bps_60s ?? null,
                                    )}
                                    icon={LineChart}
                                    trend={
                                        paperStats?.avg_markout_bps_60s
                                            ? parseFloat(
                                                  paperStats.avg_markout_bps_60s,
                                              ) > 0
                                                ? 'up'
                                                : parseFloat(
                                                        paperStats.avg_markout_bps_60s,
                                                    ) < 0
                                                  ? 'down'
                                                  : 'neutral'
                                            : undefined
                                    }
                                />
                            </div>
                        </TabsContent>
                    </Tabs>

                    <Card className="mt-6 border-l-4 border-l-emerald-500/50">
                        <CardHeader>
                            <div className="flex items-center gap-3">
                                <div className="rounded-lg bg-emerald-500/10 p-2 dark:bg-emerald-500/15">
                                    <ArrowLeftRight className="size-4 text-emerald-600 dark:text-emerald-400" />
                                </div>
                                <CardTitle>Recent Trades</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent>
                            {!recentTrades?.length ? (
                                <p className="text-sm text-muted-foreground">
                                    No trades yet.
                                </p>
                            ) : (
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>Date</TableHead>
                                            <TableHead>Market</TableHead>
                                            <TableHead>Side</TableHead>
                                            <TableHead>Outcome</TableHead>
                                            <TableHead>Ref</TableHead>
                                            <TableHead>Fill</TableHead>
                                            <TableHead>Slip</TableHead>
                                            <TableHead>
                                                <LabelWithTooltip
                                                    label="1m"
                                                    tooltip={markoutTooltipText}
                                                />
                                            </TableHead>
                                            <TableHead>Size</TableHead>
                                            <TableHead>Type</TableHead>
                                            <TableHead>Status</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {recentTrades.map((trade) => (
                                            <TableRow key={trade.id}>
                                                <TableCell className="text-muted-foreground">
                                                    {trade.executed_at ||
                                                    trade.created_at
                                                        ? new Date(
                                                              trade.executed_at ??
                                                                  trade.created_at ??
                                                                  '',
                                                          ).toLocaleDateString()
                                                        : '-'}
                                                </TableCell>
                                                <TableCell className="max-w-[200px] truncate font-mono text-xs">
                                                    {trade.symbol ?? '-'}
                                                </TableCell>
                                                <TableCell>
                                                    <span
                                                        className={
                                                            trade.side === 'buy'
                                                                ? 'text-emerald-600 dark:text-emerald-400'
                                                                : 'text-red-500 dark:text-red-400'
                                                        }
                                                    >
                                                        {trade.side?.toUpperCase() ??
                                                            '-'}
                                                    </span>
                                                </TableCell>
                                                <TableCell>
                                                    {trade.outcome ?? '-'}
                                                </TableCell>
                                                <TableCell className="tabular-nums">
                                                    {formatPrice(
                                                        trade.reference_price ??
                                                            trade.price,
                                                    )}
                                                </TableCell>
                                                <TableCell className="tabular-nums">
                                                    {formatPrice(
                                                        trade.filled_price ??
                                                            trade.resolved_price,
                                                    )}
                                                </TableCell>
                                                <TableCell
                                                    className={`tabular-nums ${bpsColorClass(trade.fill_slippage_bps, false)}`}
                                                >
                                                    {formatBps(
                                                        trade.fill_slippage_bps,
                                                    )}
                                                </TableCell>
                                                <TableCell
                                                    className={`tabular-nums ${bpsColorClass(trade.markout_bps_60s, true)}`}
                                                >
                                                    {formatBps(
                                                        trade.markout_bps_60s,
                                                    )}
                                                </TableCell>
                                                <TableCell className="tabular-nums">
                                                    {formatPnl(trade.size_usdc)}
                                                </TableCell>
                                                <TableCell>
                                                    <span
                                                        className={`inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
                                                            trade.is_paper
                                                                ? 'bg-amber-500/10 text-amber-700 dark:text-amber-400'
                                                                : 'bg-blue-500/10 text-blue-700 dark:text-blue-400'
                                                        }`}
                                                    >
                                                        {trade.is_paper
                                                            ? 'Paper'
                                                            : 'Live'}
                                                    </span>
                                                </TableCell>
                                                <TableCell>
                                                    <span
                                                        className={`inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
                                                            trade.status ===
                                                            'filled'
                                                                ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-400'
                                                                : trade.status ===
                                                                    'pending'
                                                                  ? 'bg-amber-500/10 text-amber-700 dark:text-amber-400'
                                                                  : 'bg-muted text-muted-foreground'
                                                        }`}
                                                    >
                                                        {trade.status}
                                                    </span>
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            )}
                        </CardContent>
                    </Card>
                </Deferred>

                <Card className="mt-6 border-l-4 border-l-amber-500/50">
                    <CardHeader>
                        <div className="flex items-center gap-3">
                            <div className="rounded-lg bg-amber-500/10 p-2 dark:bg-amber-500/15">
                                <LineChart className="size-4 text-amber-600 dark:text-amber-400" />
                            </div>
                            <CardTitle>Recent Backtests</CardTitle>
                        </div>
                    </CardHeader>
                    <CardContent>
                        {!strategy.backtest_results?.length ? (
                            <p className="text-sm text-muted-foreground">
                                No backtests yet.
                            </p>
                        ) : (
                            <BacktestResultsTable
                                results={strategy.backtest_results}
                            />
                        )}
                    </CardContent>
                </Card>

                <Card className="mt-6 border-l-4 border-l-cyan-500/50">
                    <CardHeader>
                        <div className="flex items-center gap-3">
                            <div className="rounded-lg bg-cyan-500/10 p-2 dark:bg-cyan-500/15">
                                <FlaskConical className="size-4 text-cyan-600 dark:text-cyan-400" />
                            </div>
                            <CardTitle>Run Backtest</CardTitle>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <form
                            onSubmit={handleBacktestSubmit}
                            className="space-y-6"
                        >
                            <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
                                <div className="space-y-2">
                                    <Label htmlFor="date_from">Date From</Label>
                                    <Input
                                        id="date_from"
                                        type="date"
                                        value={backtestForm.data.date_from}
                                        onChange={(e) =>
                                            backtestForm.setData(
                                                'date_from',
                                                e.target.value,
                                            )
                                        }
                                    />
                                    <InputError
                                        message={backtestForm.errors.date_from}
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="date_to">Date To</Label>
                                    <Input
                                        id="date_to"
                                        type="date"
                                        value={backtestForm.data.date_to}
                                        onChange={(e) =>
                                            backtestForm.setData(
                                                'date_to',
                                                e.target.value,
                                            )
                                        }
                                    />
                                    <InputError
                                        message={backtestForm.errors.date_to}
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label>Markets</Label>
                                    <div className="flex flex-wrap gap-1.5">
                                        {MARKET_OPTIONS.map((m) => {
                                            const isActive =
                                                backtestForm.data.market_filter
                                                    .length === 0 ||
                                                backtestForm.data.market_filter.includes(
                                                    m.value,
                                                );
                                            return (
                                                <button
                                                    key={m.value}
                                                    type="button"
                                                    onClick={() => {
                                                        const current =
                                                            backtestForm.data
                                                                .market_filter;
                                                        let next: string[];
                                                        if (
                                                            current.length === 0
                                                        ) {
                                                            next = [m.value];
                                                        } else if (
                                                            current.includes(
                                                                m.value,
                                                            )
                                                        ) {
                                                            next =
                                                                current.filter(
                                                                    (v) =>
                                                                        v !==
                                                                        m.value,
                                                                );
                                                        } else {
                                                            next = [
                                                                ...current,
                                                                m.value,
                                                            ];
                                                        }
                                                        backtestForm.setData(
                                                            'market_filter',
                                                            next,
                                                        );
                                                    }}
                                                    className={`rounded-md border px-2.5 py-1 text-xs font-medium transition-colors ${
                                                        isActive
                                                            ? 'border-primary bg-primary text-primary-foreground'
                                                            : 'border-border bg-background text-muted-foreground hover:bg-accent'
                                                    }`}
                                                >
                                                    {m.label}
                                                </button>
                                            );
                                        })}
                                    </div>
                                    <p className="text-xs text-muted-foreground">
                                        {backtestForm.data.market_filter
                                            .length === 0
                                            ? 'All markets'
                                            : `${backtestForm.data.market_filter.length} selected`}
                                    </p>
                                </div>
                            </div>
                            <Button
                                type="submit"
                                size="lg"
                                disabled={backtestForm.processing}
                            >
                                {backtestForm.processing
                                    ? 'Running...'
                                    : 'Run Backtest'}
                            </Button>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </AppLayout>
    );
}
