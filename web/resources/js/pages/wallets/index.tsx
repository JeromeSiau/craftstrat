import { Head, router, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [{ title: 'Wallets', href: '/wallets' }];

interface Wallet {
    id: number;
    label: string | null;
    address: string;
    balance_usdc: string;
    is_active: boolean;
    strategies_count: number;
}

export default function WalletsIndex({ wallets }: { wallets: Wallet[] }) {
    const { data, setData, post, processing, reset } = useForm({ label: '' });

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post('/wallets', { onSuccess: () => reset() });
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Wallets" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Wallets</h1>

                <form
                    onSubmit={handleSubmit}
                    className="mb-6 flex items-end gap-3"
                >
                    <div>
                        <Label htmlFor="label">Label (optional)</Label>
                        <Input
                            id="label"
                            value={data.label}
                            onChange={(e) => setData('label', e.target.value)}
                            placeholder="My trading wallet"
                        />
                    </div>
                    <Button type="submit" disabled={processing}>
                        Generate Wallet
                    </Button>
                </form>

                <div className="space-y-3">
                    {wallets.length === 0 && (
                        <p className="text-muted-foreground">
                            No wallets yet. Generate your first one above.
                        </p>
                    )}
                    {wallets.map((wallet) => (
                        <div
                            key={wallet.id}
                            className="flex items-center justify-between rounded-xl border border-sidebar-border/70 p-4 dark:border-sidebar-border"
                        >
                            <div>
                                <h3 className="font-semibold">
                                    {wallet.label || 'Unnamed Wallet'}
                                </h3>
                                <p className="font-mono text-xs text-muted-foreground">
                                    {wallet.address}
                                </p>
                                <p className="mt-1 text-sm text-muted-foreground">
                                    $
                                    {parseFloat(wallet.balance_usdc).toFixed(2)}{' '}
                                    USDC · {wallet.strategies_count} strateg
                                    {wallet.strategies_count === 1
                                        ? 'y'
                                        : 'ies'}
                                </p>
                            </div>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() =>
                                    router.delete(`/wallets/${wallet.id}`)
                                }
                            >
                                Delete
                            </Button>
                        </div>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
