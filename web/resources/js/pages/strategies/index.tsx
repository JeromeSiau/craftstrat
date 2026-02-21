import { Head, Link } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: '/strategies' },
];

interface Strategy {
    id: number;
    name: string;
    mode: string;
    is_active: boolean;
    wallets_count: number;
    created_at: string;
}

export default function StrategiesIndex({
    strategies,
}: {
    strategies: Strategy[];
}) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Strategies" />
            <div className="p-6">
                <div className="mb-6 flex items-center justify-between">
                    <h1 className="text-2xl font-bold">Strategies</h1>
                    <Link href="/strategies/create">
                        <Button>New Strategy</Button>
                    </Link>
                </div>
                <div className="space-y-3">
                    {strategies.length === 0 && (
                        <p className="text-muted-foreground">
                            No strategies yet. Create your first one.
                        </p>
                    )}
                    {strategies.map((strategy) => (
                        <Link
                            key={strategy.id}
                            href={`/strategies/${strategy.id}`}
                            className="block rounded-lg border border-sidebar-border p-4 transition hover:bg-accent"
                        >
                            <div className="flex items-center justify-between">
                                <div>
                                    <h3 className="font-semibold">
                                        {strategy.name}
                                    </h3>
                                    <p className="text-sm text-muted-foreground">
                                        {strategy.mode} mode ·{' '}
                                        {strategy.wallets_count} wallet(s)
                                    </p>
                                </div>
                                <span
                                    className={`rounded-full px-2 py-1 text-xs font-medium ${strategy.is_active ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'}`}
                                >
                                    {strategy.is_active ? 'Active' : 'Inactive'}
                                </span>
                            </div>
                        </Link>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
