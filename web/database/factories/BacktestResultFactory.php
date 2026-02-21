<?php

namespace Database\Factories;

use App\Models\Strategy;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\BacktestResult>
 */
class BacktestResultFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'strategy_id' => Strategy::factory(),
            'market_filter' => null,
            'date_from' => now()->subDays(30),
            'date_to' => now(),
            'total_trades' => fake()->numberBetween(10, 500),
            'win_rate' => fake()->randomFloat(4, 0.3, 0.8),
            'total_pnl_usdc' => fake()->randomFloat(6, -500, 5000),
            'max_drawdown' => fake()->randomFloat(4, 0.01, 0.5),
            'sharpe_ratio' => fake()->randomFloat(4, -1, 3),
            'result_detail' => null,
        ];
    }
}
