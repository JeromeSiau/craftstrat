<?php

namespace App\Services;

use Elliptic\EC;
use kornrunner\Keccak;
use RuntimeException;

class WalletService
{
    private readonly string $encryptionKey;

    public function __construct(string $encryptionKey)
    {
        if (strlen($encryptionKey) < 32) {
            throw new RuntimeException('ENCRYPTION_KEY must be at least 32 characters.');
        }

        $this->encryptionKey = $encryptionKey;
    }

    /**
     * Generate a new Ethereum/Polygon signer keypair.
     *
     * @return array{signer_address: string, private_key_enc: string}
     */
    public function generateKeypair(): array
    {
        $ec = new EC('secp256k1');
        $keyPair = $ec->genKeyPair();

        $privateKeyHex = $keyPair->getPrivate('hex');
        $publicKeyHex = substr($keyPair->getPublic(false, 'hex'), 2); // remove '04' prefix

        $signerAddress = $this->publicKeyToAddress($publicKeyHex);
        $encryptedKey = $this->encrypt($privateKeyHex);

        return [
            'signer_address' => $signerAddress,
            'private_key_enc' => $encryptedKey,
        ];
    }

    /**
     * Encrypt a private key with AES-256-GCM.
     */
    public function encrypt(string $plaintext): string
    {
        $key = substr(hash('sha256', $this->encryptionKey, true), 0, 32);
        $iv = random_bytes(12);
        $tag = '';

        $ciphertext = openssl_encrypt(
            $plaintext,
            'aes-256-gcm',
            $key,
            OPENSSL_RAW_DATA,
            $iv,
            $tag,
        );

        if ($ciphertext === false) {
            throw new RuntimeException('Encryption failed.');
        }

        return base64_encode($iv.$tag.$ciphertext);
    }

    /**
     * Decrypt a private key from AES-256-GCM.
     */
    public function decrypt(string $encrypted): string
    {
        $key = substr(hash('sha256', $this->encryptionKey, true), 0, 32);
        $decoded = base64_decode($encrypted, true);

        if ($decoded === false || strlen($decoded) < 28) {
            throw new RuntimeException('Invalid encrypted data.');
        }

        $iv = substr($decoded, 0, 12);
        $tag = substr($decoded, 12, 16);
        $ciphertext = substr($decoded, 28);

        $plaintext = openssl_decrypt(
            $ciphertext,
            'aes-256-gcm',
            $key,
            OPENSSL_RAW_DATA,
            $iv,
            $tag,
        );

        if ($plaintext === false) {
            throw new RuntimeException('Decryption failed — invalid key or corrupted data.');
        }

        return $plaintext;
    }

    /**
     * Derive an EIP-55 checksummed Ethereum address from an uncompressed public key (without 04 prefix).
     */
    private function publicKeyToAddress(string $publicKeyHex): string
    {
        $hash = Keccak::hash(hex2bin($publicKeyHex), 256);
        $addressLower = substr($hash, -40);

        return $this->toChecksumAddress($addressLower);
    }

    /**
     * Apply EIP-55 mixed-case checksum to an address.
     */
    private function toChecksumAddress(string $addressLower): string
    {
        $hash = Keccak::hash($addressLower, 256);
        $checksummed = '0x';

        for ($i = 0; $i < 40; $i++) {
            if (intval($hash[$i], 16) >= 8) {
                $checksummed .= strtoupper($addressLower[$i]);
            } else {
                $checksummed .= $addressLower[$i];
            }
        }

        return $checksummed;
    }
}
