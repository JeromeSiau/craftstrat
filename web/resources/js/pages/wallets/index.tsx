import { useState } from 'react';
import { Head, router, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import type { BreadcrumbItem } from '@/types';
import type { Wallet } from '@/types/models';
import {
    index,
    store,
    destroy,
    assignStrategy,
} from '@/actions/App/Http/Controllers/WalletController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Wallets', href: index.url() },
];

interface Props {
    wallets: { data: Wallet[] };
    strategies: Array<{ id: number; name: string }>;
}

export default function WalletsIndex({ wallets, strategies }: Props) {
    const { data, setData, post, processing, reset } = useForm({ label: '' });
    const assignForm = useForm({
        strategy_id: '',
        max_position_usdc: '100',
    });
    const [openDialogWalletId, setOpenDialogWalletId] = useState<number | null>(
        null,
    );

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post(store.url(), { onSuccess: () => reset() });
    }

    function handleAssign(walletId: number) {
        assignForm.post(assignStrategy.url(walletId), {
            onSuccess: () => {
                setOpenDialogWalletId(null);
                assignForm.reset();
            },
        });
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
                    {wallets.data.length === 0 && (
                        <p className="text-muted-foreground">
                            No wallets yet. Generate your first one above.
                        </p>
                    )}
                    {wallets.data.map((wallet) => (
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
                                    USDC &middot; {wallet.strategies_count}{' '}
                                    strateg
                                    {wallet.strategies_count === 1
                                        ? 'y'
                                        : 'ies'}
                                </p>
                            </div>
                            <div className="flex items-center gap-2">
                                <Dialog
                                    open={openDialogWalletId === wallet.id}
                                    onOpenChange={(open) =>
                                        setOpenDialogWalletId(
                                            open ? wallet.id : null,
                                        )
                                    }
                                >
                                    <DialogTrigger asChild>
                                        <Button variant="outline" size="sm">
                                            Assign Strategy
                                        </Button>
                                    </DialogTrigger>
                                    <DialogContent>
                                        <DialogHeader>
                                            <DialogTitle>
                                                Assign Strategy to{' '}
                                                {wallet.label ||
                                                    'Unnamed Wallet'}
                                            </DialogTitle>
                                        </DialogHeader>
                                        <div className="space-y-4 pt-2">
                                            <div>
                                                <Label htmlFor="strategy_id">
                                                    Strategy
                                                </Label>
                                                <Select
                                                    value={
                                                        assignForm.data
                                                            .strategy_id
                                                    }
                                                    onValueChange={(value) =>
                                                        assignForm.setData(
                                                            'strategy_id',
                                                            value,
                                                        )
                                                    }
                                                >
                                                    <SelectTrigger
                                                        id="strategy_id"
                                                        className="mt-1"
                                                    >
                                                        <SelectValue placeholder="Select a strategy" />
                                                    </SelectTrigger>
                                                    <SelectContent>
                                                        {strategies.map(
                                                            (strategy) => (
                                                                <SelectItem
                                                                    key={
                                                                        strategy.id
                                                                    }
                                                                    value={String(
                                                                        strategy.id,
                                                                    )}
                                                                >
                                                                    {
                                                                        strategy.name
                                                                    }
                                                                </SelectItem>
                                                            ),
                                                        )}
                                                    </SelectContent>
                                                </Select>
                                                {assignForm.errors
                                                    .strategy_id && (
                                                    <p className="mt-1 text-sm text-destructive">
                                                        {
                                                            assignForm.errors
                                                                .strategy_id
                                                        }
                                                    </p>
                                                )}
                                            </div>
                                            <div>
                                                <Label htmlFor="max_position_usdc">
                                                    Max Position (USDC)
                                                </Label>
                                                <Input
                                                    id="max_position_usdc"
                                                    type="number"
                                                    min="1"
                                                    className="mt-1"
                                                    value={
                                                        assignForm.data
                                                            .max_position_usdc
                                                    }
                                                    onChange={(e) =>
                                                        assignForm.setData(
                                                            'max_position_usdc',
                                                            e.target.value,
                                                        )
                                                    }
                                                />
                                                {assignForm.errors
                                                    .max_position_usdc && (
                                                    <p className="mt-1 text-sm text-destructive">
                                                        {
                                                            assignForm.errors
                                                                .max_position_usdc
                                                        }
                                                    </p>
                                                )}
                                            </div>
                                            <Button
                                                onClick={() =>
                                                    handleAssign(wallet.id)
                                                }
                                                disabled={
                                                    assignForm.processing ||
                                                    !assignForm.data.strategy_id
                                                }
                                                className="w-full"
                                            >
                                                Assign
                                            </Button>
                                        </div>
                                    </DialogContent>
                                </Dialog>
                                <Button
                                    variant="destructive"
                                    size="sm"
                                    onClick={() => {
                                        if (
                                            confirm(
                                                'Are you sure you want to delete this wallet? This action cannot be undone.',
                                            )
                                        ) {
                                            router.delete(
                                                destroy.url(wallet.id),
                                            );
                                        }
                                    }}
                                >
                                    Delete
                                </Button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
