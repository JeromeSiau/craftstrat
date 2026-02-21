<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('copy_trades', function (Blueprint $table) {
            $table->id();
            $table->foreignId('copy_relationship_id')->constrained();
            $table->foreignId('follower_trade_id')->nullable()->constrained('trades');
            $table->string('leader_address');
            $table->string('leader_market_id', 100)->nullable();
            $table->string('leader_outcome', 10)->nullable();
            $table->decimal('leader_price', 10, 6)->nullable();
            $table->decimal('leader_size_usdc', 18, 6)->nullable();
            $table->string('leader_tx_hash')->nullable();
            $table->decimal('follower_price', 10, 6)->nullable();
            $table->decimal('slippage_pct', 8, 6)->nullable();
            $table->string('status', 20)->default('pending');
            $table->string('skip_reason')->nullable();
            $table->timestamp('detected_at');
            $table->timestamp('executed_at')->nullable();
            $table->timestamp('created_at')->useCurrent();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('copy_trades');
    }
};
