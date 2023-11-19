#enc.py
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import padding
import os

def encrypt_image(input_path, encrypted_image_path, key):
    # Read the image file
    with open(input_path, 'rb') as file:
        plaintext = file.read()

    # Pad the plaintext to match the block size of the chosen cipher
    padder = padding.PKCS7(algorithms.AES.block_size).padder()
    plaintext_padded = padder.update(plaintext) + padder.finalize()

    # Generate a random IV (Initialization Vector)
    iv = os.urandom(algorithms.AES.block_size // 8)

    # Create an AES cipher object
    cipher = Cipher(algorithms.AES(key), modes.CFB(iv), backend=default_backend())

    # Encrypt the plaintext
    encryptor = cipher.encryptor()
    ciphertext = encryptor.update(plaintext_padded) + encryptor.finalize()

    # Write the IV and ciphertext to the output file
    with open(encrypted_image_path, 'wb') as file:
        file.write(iv + ciphertext)

    # Save the key to a separate file
    key_file_path = encrypted_image_path + '.key'
    with open(key_file_path, 'wb') as key_file:
        key_file.write(key)

# Get the directory of the script
script_directory = os.path.dirname(os.path.abspath(__file__))

# Set the key file path
key_file = os.path.join(script_directory, 'your_key_file.key')

# Generate or load the key
if os.path.exists(key_file):
    # Load the key if it exists
    with open(key_file, 'rb') as file:
        secret_key = file.read()
else:
    # Generate a new key if it doesn't exist
    secret_key = os.urandom(16)  # 16 bytes = 128 bits
    # Save the key to a file
    with open(key_file, 'wb') as file:
        file.write(secret_key)

# Encrypt all .png images in the directory, excluding 'cover.png'
for filename in os.listdir(script_directory):
    if filename.endswith('.jpg') and filename != 'cover.jpg':
        input_image_path = os.path.join(script_directory, filename)
        encrypted_image_path = os.path.join(script_directory, f'{os.path.splitext(filename)[0]}.enc')
        encrypt_image(input_image_path, encrypted_image_path, secret_key)
