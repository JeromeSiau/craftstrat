<?php

namespace Database\Factories;

use App\Models\Strategy;
use App\Models\Wallet;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Trade>
 */
class TradeFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'wallet_id' => Wallet::factory(),
            'strategy_id' => Strategy::factory(),
            'symbol' => 'btc-updown-15m-'.fake()->unixTime(),
            'side' => fake()->randomElement(['buy', 'sell']),
            'outcome' => fake()->randomElement(['UP', 'DOWN']),
            'price' => fake()->randomFloat(6, 0.01, 0.99),
            'reference_price' => fake()->randomFloat(6, 0.01, 0.99),
            'size_usdc' => fake()->randomFloat(6, 10, 500),
            'order_type' => fake()->randomElement(['market', 'limit']),
            'status' => 'filled',
            'fee_bps' => fake()->randomElement([0, 50, 100]),
            'filled_price' => fake()->randomFloat(6, 0.01, 0.99),
            'resolved_price' => null,
            'fill_slippage_bps' => fake()->randomFloat(2, -25, 25),
            'fill_slippage_pct' => fake()->randomFloat(6, -0.01, 0.01),
            'markout_price_60s' => fake()->randomFloat(6, 0.01, 0.99),
            'markout_at_60s' => now(),
            'markout_bps_60s' => fake()->randomFloat(2, -50, 50),
            'executed_at' => now(),
        ];
    }

    /**
     * Indicate that the trade is pending.
     */
    public function pending(): static
    {
        return $this->state(fn () => ['status' => 'pending', 'executed_at' => null]);
    }
}
