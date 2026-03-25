<?php

namespace App\Http\Controllers;

use App\Models\WalletStrategy;
use App\Services\TradePerformanceService;
use Inertia\Inertia;
use Inertia\Response;

class DashboardController extends Controller
{
    public function index(TradePerformanceService $performance): Response
    {
        $user = auth()->user();
        $walletIds = $user->wallets()->pluck('id')->all();

        return Inertia::render('dashboard', [
            'stats' => [
                'active_strategies' => $user->strategies()->where('is_active', true)->count(),
                'total_strategies' => $user->strategies()->count(),
                'total_wallets' => $user->wallets()->count(),
                'total_pnl_usdc' => $performance->totalPnlForWalletIds($walletIds),
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
