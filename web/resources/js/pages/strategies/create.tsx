import { Head, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: '/strategies' },
    { title: 'Create', href: '/strategies/create' },
];

export default function StrategiesCreate() {
    const { data, setData, post, processing, errors } = useForm({
        name: '',
        description: '',
        mode: 'form',
        graph: {
            mode: 'form',
            conditions: [],
            action: {
                signal: 'buy',
                outcome: 'UP',
                size_mode: 'fixed',
                size_usdc: 50,
                order_type: 'market',
            },
            risk: {
                stoploss_pct: 30,
                take_profit_pct: 80,
                max_position_usdc: 200,
                max_trades_per_slot: 1,
            },
        },
    });

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post('/strategies');
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Create Strategy" />
            <div className="mx-auto max-w-2xl p-6">
                <h1 className="mb-6 text-2xl font-bold">Create Strategy</h1>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <Label htmlFor="name">Name</Label>
                        <Input
                            id="name"
                            value={data.name}
                            onChange={(e) => setData('name', e.target.value)}
                        />
                        {errors.name && (
                            <p className="mt-1 text-sm text-red-500">
                                {errors.name}
                            </p>
                        )}
                    </div>
                    <div>
                        <Label htmlFor="description">Description</Label>
                        <Input
                            id="description"
                            value={data.description}
                            onChange={(e) =>
                                setData('description', e.target.value)
                            }
                        />
                    </div>
                    <p className="text-sm text-muted-foreground">
                        Strategy builder will be available in a future update. A
                        default strategy configuration is used for now.
                    </p>
                    <Button type="submit" disabled={processing}>
                        Create Strategy
                    </Button>
                </form>
            </div>
        </AppLayout>
    );
}
