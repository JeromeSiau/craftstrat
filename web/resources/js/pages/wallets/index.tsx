import { useState } from 'react';
import { Head, router, useForm } from '@inertiajs/react';
import { KeyRound, Plus, Wallet as WalletIcon } from 'lucide-react';
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
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">Wallets</h1>
                    <p className="mt-1 text-muted-foreground">
                        Generate and manage your trading wallets.
                    </p>
                </div>

                <Card className="mb-8 border-l-4 border-l-emerald-500/50">
                    <CardContent className="pt-6">
                        <div className="mb-4 flex items-center gap-3">
                            <div className="rounded-lg bg-emerald-500/10 p-2 dark:bg-emerald-500/15">
                                <KeyRound className="size-4 text-emerald-600 dark:text-emerald-400" />
                            </div>
                            <div>
                                <h3 className="font-semibold">Generate Wallet</h3>
                                <p className="text-sm text-muted-foreground">Create a new trading wallet.</p>
                            </div>
                        </div>
                        <form onSubmit={handleSubmit} className="flex flex-col gap-4 sm:flex-row sm:items-end">
                            <div className="flex-1 space-y-2">
                                <Label htmlFor="label">Wallet Label (optional)</Label>
                                <Input
                                    id="label"
                                    value={data.label}
                                    onChange={(e) => setData('label', e.target.value)}
                                    placeholder="e.g. BTC Long Strategy"
                                />
                            </div>
                            <Button type="submit" size="lg" disabled={processing}>
                                <Plus className="size-4" />
                                Generate Wallet
                            </Button>
                        </form>
                    </CardContent>
                </Card>

                {wallets.data.length === 0 ? (
                    <Card>
                        <CardContent className="flex flex-col items-center justify-center py-16 text-center">
                            <div className="rounded-xl bg-muted p-4">
                                <WalletIcon className="size-8 text-muted-foreground" />
                            </div>
                            <p className="mt-4 font-medium">No wallets yet</p>
                            <p className="mt-1 text-sm text-muted-foreground">
                                Generate your first wallet using the form above.
                            </p>
                        </CardContent>
                    </Card>
                ) : (
                    <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                        {wallets.data.map((wallet) => (
                            <Card key={wallet.id} className="border-l-4 border-l-violet-500/50">
                                <CardContent className="py-5">
                                    <div className="mb-4">
                                        <h3 className="truncate font-semibold">
                                            {wallet.label || 'Unnamed Wallet'}
                                        </h3>
                                        <p className="mt-1 truncate font-mono text-xs text-muted-foreground">
                                            {wallet.address}
                                        </p>
                                    </div>
                                    <div className="mb-4 flex gap-4 text-sm">
                                        <div className="rounded-md bg-muted/50 px-3 py-1.5">
                                            <span className="font-semibold tabular-nums">
                                                ${parseFloat(wallet.balance_usdc).toFixed(2)}
                                            </span>
                                            <span className="ml-1 text-muted-foreground">USDC</span>
                                        </div>
                                        <div className="rounded-md bg-muted/50 px-3 py-1.5">
                                            <span className="font-semibold tabular-nums">
                                                {wallet.strategies_count ?? 0}
                                            </span>
                                            <span className="ml-1 text-muted-foreground">
                                                strateg{(wallet.strategies_count ?? 0) === 1 ? 'y' : 'ies'}
                                            </span>
                                        </div>
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
                )}
            </div>
        </AppLayout>
    );
}
