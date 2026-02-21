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
        Schema::create('watched_wallets', function (Blueprint $table) {
            $table->id();
            $table->string('address')->unique();
            $table->string('label')->nullable();
            $table->integer('follower_count')->default(0);
            $table->decimal('win_rate', 5, 4)->nullable();
            $table->decimal('total_pnl_usdc', 18, 6)->nullable();
            $table->decimal('avg_slippage', 8, 6)->nullable();
            $table->timestamp('last_seen_at')->nullable();
            $table->timestamp('updated_at')->useCurrent();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('watched_wallets');
    }
};
