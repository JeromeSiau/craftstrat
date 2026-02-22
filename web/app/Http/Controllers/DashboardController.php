<?php

namespace App\Http\Controllers;

use App\Models\Trade;
use App\Models\WalletStrategy;
use Inertia\Inertia;
use Inertia\Response;

class DashboardController extends Controller
{
    public function index(): Response
    {
        $user = auth()->user();
        $walletIds = $user->wallets()->pluck('id');

        return Inertia::render('dashboard', [
            'stats' => [
                'active_strategies' => $user->strategies()->where('is_active', true)->count(),
                'total_strategies' => $user->strategies()->count(),
                'total_wallets' => $user->wallets()->count(),
                'total_pnl_usdc' => Trade::whereIn('wallet_id', $walletIds)
                    ->where('status', 'filled')
                    ->sum('size_usdc'),
                'running_assignments' => WalletStrategy::whereIn('wallet_id', $walletIds)
                    ->where('is_running', true)
                    ->count(),
            ],
            'recentStrategies' => $user->strategies()
                ->withCount('wallets')
                ->latest()
                ->limit(5)
                ->get(),
        ]);
    }
}
