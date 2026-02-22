import { Head, Link } from '@inertiajs/react';
import { Activity, LineChart, Target, Wallet } from 'lucide-react';
import AppLayout from '@/layouts/app-layout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import MetricCard from '@/components/metric-card';
import StatusBadge from '@/components/status-badge';
import type { BreadcrumbItem } from '@/types';
import type { DashboardStats, Strategy } from '@/types/models';
import { dashboard } from '@/routes';
import { show as strategyShow } from '@/actions/App/Http/Controllers/StrategyController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Dashboard', href: dashboard().url },
];

interface Props {
    stats: DashboardStats;
    recentStrategies: Strategy[];
}

export default function Dashboard({ stats, recentStrategies }: Props) {
    const cards = [
        { label: 'Active Strategies', value: stats.active_strategies, icon: Target },
        { label: 'Total Wallets', value: stats.total_wallets, icon: Wallet },
        { label: 'Running Assignments', value: stats.running_assignments, icon: Activity },
        {
            label: 'Total PnL',
            value: `$${parseFloat(stats.total_pnl_usdc || '0').toFixed(2)}`,
            icon: LineChart,
        },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Dashboard" />
            <div className="flex flex-1 flex-col gap-6 p-6">
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    {cards.map((card) => (
                        <MetricCard
                            key={card.label}
                            label={card.label}
                            value={card.value}
                            icon={card.icon}
                        />
                    ))}
                </div>

                <Card>
                    <CardHeader>
                        <CardTitle>Recent Strategies</CardTitle>
                    </CardHeader>
                    <CardContent>
                        {recentStrategies.length === 0 ? (
                            <p className="text-sm text-muted-foreground">No strategies yet.</p>
                        ) : (
                            <div className="space-y-3">
                                {recentStrategies.map((strategy) => (
                                    <Link
                                        key={strategy.id}
                                        href={strategyShow.url(strategy.id)}
                                        className="flex items-center justify-between rounded-lg border p-3 transition hover:bg-accent"
                                    >
                                        <div>
                                            <p className="font-medium">{strategy.name}</p>
                                            <p className="text-sm text-muted-foreground">
                                                {strategy.mode} mode · {strategy.wallets_count ?? 0} wallet(s)
                                            </p>
                                        </div>
                                        <StatusBadge active={strategy.is_active} />
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
