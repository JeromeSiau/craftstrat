<?php

namespace Database\Factories;

use App\Models\CopyRelationship;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\CopyTrade>
 */
class CopyTradeFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'copy_relationship_id' => CopyRelationship::factory(),
            'leader_address' => '0x'.fake()->regexify('[a-fA-F0-9]{40}'),
            'leader_market_id' => 'btc-updown-15m-'.fake()->unixTime(),
            'leader_outcome' => fake()->randomElement(['UP', 'DOWN']),
            'leader_price' => fake()->randomFloat(6, 0.01, 0.99),
            'leader_size_usdc' => fake()->randomFloat(6, 10, 500),
            'leader_tx_hash' => '0x'.fake()->sha256(),
            'follower_price' => fake()->randomFloat(6, 0.01, 0.99),
            'slippage_pct' => fake()->randomFloat(6, -0.05, 0.05),
            'status' => 'filled',
            'detected_at' => now(),
            'executed_at' => now(),
        ];
    }
}
