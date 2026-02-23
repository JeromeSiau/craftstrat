<?php

namespace App\Http\Controllers;

use App\Models\Wallet;
use App\Notifications\StrategyAlertNotification;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class InternalNotificationController extends Controller
{
    public function send(Request $request): JsonResponse
    {
        $validated = $request->validate([
            'wallet_id' => ['required', 'integer'],
            'strategy_name' => ['required', 'string'],
            'message' => ['required', 'string', 'max:1000'],
            'channel' => ['required', 'string', 'in:database,mail'],
        ]);

        $wallet = Wallet::find($validated['wallet_id']);

        if (! $wallet) {
            return response()->json(['error' => 'Wallet not found'], 404);
        }

        $user = $wallet->user;

        if (! $user) {
            return response()->json(['error' => 'User not found'], 404);
        }

        $user->notify(new StrategyAlertNotification(
            strategyName: $validated['strategy_name'],
            message: $validated['message'],
            channel: $validated['channel'],
        ));

        return response()->json(['status' => 'sent']);
    }
}
