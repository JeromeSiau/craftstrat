<?php

use App\Services\WalletService;

beforeEach(function () {
    $this->service = new WalletService(
        encryptionKey: 'test-encryption-key-at-least-32-chars-long',
    );
});

it('generates a valid ethereum keypair', function () {
    $result = $this->service->generateKeypair();

    expect($result)->toHaveKeys(['signer_address', 'private_key_enc'])
        ->and($result['signer_address'])->toStartWith('0x')
        ->and($result['signer_address'])->toHaveLength(42)
        ->and($result['private_key_enc'])->not->toBeEmpty();
});

it('generates unique addresses on each call', function () {
    $first = $this->service->generateKeypair();
    $second = $this->service->generateKeypair();

    expect($first['signer_address'])->not->toBe($second['signer_address']);
});

it('encrypts and decrypts a private key correctly', function () {
    $original = 'aabbccddee11223344556677889900aabbccddee11223344556677889900aabb';

    $encrypted = $this->service->encrypt($original);
    $decrypted = $this->service->decrypt($encrypted);

    expect($decrypted)->toBe($original)
        ->and($encrypted)->not->toBe($original);
});

it('can decrypt keys from generated keypair', function () {
    $keypair = $this->service->generateKeypair();
    $decryptedKey = $this->service->decrypt($keypair['private_key_enc']);

    expect($decryptedKey)->toHaveLength(64) // 32 bytes in hex
        ->and($decryptedKey)->toMatch('/^[a-f0-9]{64}$/');
});

it('throws on decryption with wrong key', function () {
    $encrypted = $this->service->encrypt('secret');

    $wrongService = new WalletService(
        encryptionKey: 'different-encryption-key-also-32-chars-long!!',
    );

    $wrongService->decrypt($encrypted);
})->throws(RuntimeException::class, 'Decryption failed');

it('throws when encryption key is too short', function () {
    new WalletService(encryptionKey: 'short');
})->throws(RuntimeException::class, 'ENCRYPTION_KEY must be at least 32 characters');
