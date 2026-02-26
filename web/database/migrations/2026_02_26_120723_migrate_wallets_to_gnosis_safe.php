<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Clean slate: remove all existing EOA wallets and their pivot rows
        DB::table('wallet_strategies')->truncate();
        DB::table('wallets')->truncate();

        Schema::table('wallets', function (Blueprint $table) {
            $table->dropUnique(['address']);
            $table->renameColumn('address', 'signer_address');
        });

        Schema::table('wallets', function (Blueprint $table) {
            $table->unique('signer_address');
            $table->string('safe_address')->nullable()->unique()->after('signer_address');
            $table->string('status', 20)->default('pending')->after('safe_address');
            $table->timestamp('deployed_at')->nullable()->after('is_active');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('wallets', function (Blueprint $table) {
            $table->dropUnique(['signer_address']);
            $table->dropUnique(['safe_address']);
            $table->dropColumn(['safe_address', 'status', 'deployed_at']);
            $table->renameColumn('signer_address', 'address');
        });

        Schema::table('wallets', function (Blueprint $table) {
            $table->unique('address');
        });
    }
};
