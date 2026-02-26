import { Head, router, useForm } from '@inertiajs/react';
import { Check, Copy, ExternalLink, KeyRound, Plus, RefreshCw, TriangleAlert, Wallet as WalletIcon } from 'lucide-react';
import { useState } from 'react';
import { destroy, index, retryDeploy, store } from '@/actions/App/Http/Controllers/WalletController';
import ConfirmDialog from '@/components/confirm-dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Spinner } from '@/components/ui/spinner';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import AssignStrategyDialog from '@/components/wallet/assign-strategy-dialog';
import { useClipboard } from '@/hooks/use-clipboard';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';
import type { Paginated, Wallet } from '@/types/models';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Wallets', href: index.url() },
];

interface Props {
    wallets: Paginated<Wallet>;
    strategies: Array<{ id: number; name: string }>;
}

function StatusBadge({ status }: { status: Wallet['status'] }) {
    switch (status) {
        case 'deployed':
            return <Badge className="bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border-emerald-500/25">Ready</Badge>;
        case 'pending':
        case 'deploying':
            return (
                <Badge className="animate-pulse bg-amber-500/15 text-amber-600 dark:text-amber-400 border-amber-500/25">
                    <Spinner className="size-3" />
                    Deploying Safe...
                </Badge>
            );
        case 'failed':
            return (
                <Badge className="bg-red-500/15 text-red-600 dark:text-red-400 border-red-500/25">
                    <TriangleAlert className="size-3" />
                    Failed
                </Badge>
            );
    }
}

function CopyableAddress({ address, label }: { address: string; label: string }) {
    const [copiedText, copy] = useClipboard();
    const isCopied = copiedText === address;

    return (
        <div className="space-y-1">
            <p className="text-xs text-muted-foreground">{label}</p>
            <div className="flex items-center gap-1.5">
                <code className="truncate text-xs">{address}</code>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <button
                            onClick={() => copy(address)}
                            className="shrink-0 rounded p-0.5 text-muted-foreground hover:text-foreground"
                        >
                            {isCopied ? <Check className="size-3.5 text-emerald-500" /> : <Copy className="size-3.5" />}
                        </button>
                    </TooltipTrigger>
                    <TooltipContent>{isCopied ? 'Copied!' : 'Copy address'}</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger asChild>
                        <a
                            href={`https://polygonscan.com/address/${address}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="shrink-0 rounded p-0.5 text-muted-foreground hover:text-foreground"
                        >
                            <ExternalLink className="size-3.5" />
                        </a>
                    </TooltipTrigger>
                    <TooltipContent>View on Polygonscan</TooltipContent>
                </Tooltip>
            </div>
        </div>
    );
}

function WalletCard({
    wallet,
    strategies,
    openDialogWalletId,
    setOpenDialogWalletId,
}: {
    wallet: Wallet;
    strategies: Array<{ id: number; name: string }>;
    openDialogWalletId: number | null;
    setOpenDialogWalletId: (id: number | null) => void;
}) {
    const isDeployed = wallet.status === 'deployed';
    const isDeploying = wallet.status === 'pending' || wallet.status === 'deploying';
    const isFailed = wallet.status === 'failed';

    return (
        <Card className="border-l-4 border-l-violet-500/50">
            <CardContent className="py-5">
                <div className="mb-3 flex items-start justify-between gap-2">
                    <h3 className="truncate font-semibold">
                        {wallet.label || 'Unnamed Wallet'}
                    </h3>
                    <StatusBadge status={wallet.status} />
                </div>

                {isDeployed && wallet.safe_address && (
                    <div className="mb-4 space-y-3">
                        <CopyableAddress address={wallet.safe_address} label="Safe Address (send USDC here)" />

                        <div className="rounded-md border border-amber-500/20 bg-amber-500/5 px-3 py-2">
                            <p className="text-xs font-medium text-amber-600 dark:text-amber-400">
                                Fund this wallet by sending USDC on Polygon to the Safe address above.
                            </p>
                            <p className="mt-0.5 text-xs text-muted-foreground">
                                Only send USDC on Polygon network. Other tokens or networks will result in loss of funds.
                            </p>
                        </div>
                    </div>
                )}

                {isDeploying && (
                    <p className="mb-4 text-sm text-muted-foreground">
                        Your Gnosis Safe wallet is being deployed on Polygon. This usually takes a minute.
                    </p>
                )}

                {isFailed && (
                    <p className="mb-4 text-sm text-red-500">
                        Safe deployment failed. You can retry or delete this wallet.
                    </p>
                )}

                {isDeployed && (
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
                )}

                <div className="flex items-center gap-2">
                    {isFailed && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => router.post(retryDeploy.url(wallet.id))}
                        >
                            <RefreshCw className="size-3.5" />
                            Retry Deploy
                        </Button>
                    )}
                    {isDeployed && (
                        <AssignStrategyDialog
                            walletId={wallet.id}
                            walletLabel={wallet.label}
                            strategies={strategies}
                            open={openDialogWalletId === wallet.id}
                            onOpenChange={(open) =>
                                setOpenDialogWalletId(open ? wallet.id : null)
                            }
                        />
                    )}
                    {!isDeploying && (
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
                    )}
                </div>
            </CardContent>
        </Card>
    );
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
                        Create and manage your Gnosis Safe trading wallets.
                    </p>
                </div>

                <Card className="mb-8 border-l-4 border-l-emerald-500/50">
                    <CardContent className="pt-6">
                        <div className="mb-4 flex items-center gap-3">
                            <div className="rounded-lg bg-emerald-500/10 p-2 dark:bg-emerald-500/15">
                                <KeyRound className="size-4 text-emerald-600 dark:text-emerald-400" />
                            </div>
                            <div>
                                <h3 className="font-semibold">Create Wallet</h3>
                                <p className="text-sm text-muted-foreground">Deploy a new Gnosis Safe wallet on Polygon for gas-free trading.</p>
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
                                Create Wallet
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
                                Create your first wallet using the form above.
                            </p>
                        </CardContent>
                    </Card>
                ) : (
                    <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                        {wallets.data.map((wallet) => (
                            <WalletCard
                                key={wallet.id}
                                wallet={wallet}
                                strategies={strategies}
                                openDialogWalletId={openDialogWalletId}
                                setOpenDialogWalletId={setOpenDialogWalletId}
                            />
                        ))}
                    </div>
                )}
            </div>
        </AppLayout>
    );
}
