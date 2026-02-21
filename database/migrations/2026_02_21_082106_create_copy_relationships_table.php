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
        Schema::create('copy_relationships', function (Blueprint $table) {
            $table->id();
            $table->foreignId('follower_wallet_id')->constrained('wallets')->cascadeOnDelete();
            $table->foreignId('watched_wallet_id')->constrained('watched_wallets')->cascadeOnDelete();
            $table->string('size_mode', 20)->default('proportional');
            $table->decimal('size_value', 18, 6);
            $table->decimal('max_position_usdc', 18, 6)->default(100);
            $table->jsonb('markets_filter')->nullable();
            $table->boolean('is_active')->default(true);
            $table->timestamp('created_at')->useCurrent();
            $table->unique(['follower_wallet_id', 'watched_wallet_id']);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('copy_relationships');
    }
};
