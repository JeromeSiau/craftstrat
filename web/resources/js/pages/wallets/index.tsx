import { useState } from 'react';
import { Head, router, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import ConfirmDialog from '@/components/confirm-dialog';
import AssignStrategyDialog from '@/components/wallet/assign-strategy-dialog';
import type { BreadcrumbItem } from '@/types';
import type { Wallet, Paginated } from '@/types/models';
import { index, store, destroy } from '@/actions/App/Http/Controllers/WalletController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Wallets', href: index.url() },
];

interface Props {
    wallets: Paginated<Wallet>;
    strategies: Array<{ id: number; name: string }>;
}

export default function WalletsIndex({ wallets, strategies }: Props) {
    const { data, setData, post, processing, reset } = useForm({ label: '' });
    const [openDialogWalletId, setOpenDialogWalletId] = useState<number | null>(null);

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post(store.url(), { onSuccess: () => reset() });
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Wallets" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Wallets</h1>

                <form onSubmit={handleSubmit} className="mb-6 flex items-end gap-3">
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
                    {wallets.data.length === 0 && (
                        <p className="text-muted-foreground">
                            No wallets yet. Generate your first one above.
                        </p>
                    )}
                    {wallets.data.map((wallet) => (
                        <Card key={wallet.id}>
                            <CardContent className="flex items-center justify-between py-4">
                                <div>
                                    <h3 className="font-semibold">
                                        {wallet.label || 'Unnamed Wallet'}
                                    </h3>
                                    <p className="font-mono text-xs text-muted-foreground">
                                        {wallet.address}
                                    </p>
                                    <p className="mt-1 text-sm text-muted-foreground">
                                        ${parseFloat(wallet.balance_usdc).toFixed(2)} USDC
                                        {' '}&middot;{' '}
                                        {wallet.strategies_count ?? 0}{' '}
                                        strateg{(wallet.strategies_count ?? 0) === 1 ? 'y' : 'ies'}
                                    </p>
                                </div>
                                <div className="flex items-center gap-2">
                                    <AssignStrategyDialog
                                        walletId={wallet.id}
                                        walletLabel={wallet.label}
                                        strategies={strategies}
                                        open={openDialogWalletId === wallet.id}
                                        onOpenChange={(open) =>
                                            setOpenDialogWalletId(open ? wallet.id : null)
                                        }
                                    />
                                    <ConfirmDialog
                                        trigger={
                                            <Button variant="destructive" size="sm">
                                                Delete
                                            </Button>
                                        }
                                        title="Delete Wallet"
                                        description="Are you sure you want to delete this wallet? This action cannot be undone."
                                        confirmLabel="Delete"
                                        onConfirm={() => router.delete(destroy.url(wallet.id))}
                                    />
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
