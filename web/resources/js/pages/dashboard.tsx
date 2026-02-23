import { Head, Link } from '@inertiajs/react';
import { Activity, ChevronRight, LineChart, Target, Wallet } from 'lucide-react';
import { show as strategyShow } from '@/actions/App/Http/Controllers/StrategyController';
import MetricCard from '@/components/metric-card';
import StatusBadge from '@/components/status-badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import AppLayout from '@/layouts/app-layout';
import { dashboard } from '@/routes';
import type { BreadcrumbItem } from '@/types';
import type { DashboardStats, Strategy } from '@/types/models';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Dashboard', href: dashboard().url },
];

interface Props {
    stats: DashboardStats;
    recentStrategies: Strategy[];
}

export default function Dashboard({ stats, recentStrategies }: Props) {
    const pnlValue = parseFloat(stats.total_pnl_usdc || '0');

    const cards = [
        { label: 'Active Strategies', value: stats.active_strategies, icon: Target },
        { label: 'Total Wallets', value: stats.total_wallets, icon: Wallet },
        { label: 'Running Assignments', value: stats.running_assignments, icon: Activity },
        {
            label: 'Total PnL',
            value: `$${pnlValue.toFixed(2)}`,
            icon: LineChart,
            trend: (pnlValue > 0 ? 'up' : pnlValue < 0 ? 'down' : 'neutral') as 'up' | 'down' | 'neutral',
        },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Dashboard" />
            <div className="flex flex-1 flex-col gap-6 p-4 md:p-8">
                <div>
                    <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
                    <p className="mt-1 text-muted-foreground">Overview of your trading activity.</p>
                </div>

                <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                    {cards.map((card) => (
                        <MetricCard
                            key={card.label}
                            label={card.label}
                            value={card.value}
                            icon={card.icon}
                            trend={card.trend}
                        />
                    ))}
                </div>

                <Card className="border-l-4 border-l-blue-500/50">
                    <CardHeader>
                        <div className="flex items-center gap-3">
                            <div className="rounded-lg bg-blue-500/10 p-2 dark:bg-blue-500/15">
                                <Target className="size-4 text-blue-600 dark:text-blue-400" />
                            </div>
                            <CardTitle>Recent Strategies</CardTitle>
                        </div>
                    </CardHeader>
                    <CardContent>
                        {recentStrategies.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-12 text-center">
                                <div className="rounded-xl bg-muted p-4">
                                    <Target className="size-8 text-muted-foreground" />
                                </div>
                                <p className="mt-4 font-medium">No strategies yet</p>
                                <p className="mt-1 text-sm text-muted-foreground">
                                    Create your first strategy to get started.
                                </p>
                            </div>
                        ) : (
                            <div className="divide-y">
                                {recentStrategies.map((strategy) => (
                                    <Link
                                        key={strategy.id}
                                        href={strategyShow.url(strategy.id)}
                                        className="flex items-center justify-between gap-4 py-3.5 transition first:pt-0 last:pb-0 hover:opacity-75"
                                    >
                                        <div className="min-w-0">
                                            <p className="truncate font-medium">{strategy.name}</p>
                                            <p className="mt-0.5 text-sm text-muted-foreground">
                                                {strategy.mode} mode · {strategy.wallets_count ?? 0} wallet(s)
                                            </p>
                                        </div>
                                        <div className="flex shrink-0 items-center gap-3">
                                            <StatusBadge active={strategy.is_active} />
                                            <ChevronRight className="size-4 text-muted-foreground" />
                                        </div>
                                    </Link>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>
            </div>
        </AppLayout>
    );
}
