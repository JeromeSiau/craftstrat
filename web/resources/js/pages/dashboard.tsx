import { Head, Link } from '@inertiajs/react';
import { Activity, LineChart, Target, Wallet } from 'lucide-react';
import AppLayout from '@/layouts/app-layout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
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
                        <Card key={card.label}>
                            <CardHeader className="flex flex-row items-center justify-between pb-2">
                                <CardTitle className="text-sm font-medium text-muted-foreground">
                                    {card.label}
                                </CardTitle>
                                <card.icon className="size-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <p className="text-2xl font-bold">{card.value}</p>
                            </CardContent>
                        </Card>
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
                                        <span
                                            className={`rounded-full px-2 py-1 text-xs font-medium ${
                                                strategy.is_active
                                                    ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                                                    : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
                                            }`}
                                        >
                                            {strategy.is_active ? 'Active' : 'Inactive'}
                                        </span>
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
