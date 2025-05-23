// internal crates
use crate::crypt::errors::{
    ConvertPrivateKeyToPEMErr, CryptErr, CryptFileSysErr, GenerateRSAKeyPairErr, RSAToPKeyErr,
    ReadKeyErr, SignDataErr, VerifyDataErr,
};
use crate::filesys::file::File;
use crate::filesys::path::PathExt;
use crate::trace;

// external libraries
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use secrecy::ExposeSecret;

/// Generate an RSA key pair and write the private and public keys to the specified
/// files. If the files exists, an error is returned. Files are returned instead of
/// variables holding the keys to avoid keeping sensitive information in memory. In
/// general you shouldn't interact directly with the keys but let individual functions
/// read and write to their respective files so their existence in memory is as brief as
/// possible. The public key technically doesn't need such security measures since it
/// can be shared publicly, but it's simpler to treat both keys the same. The private
/// key file is given read/write permissions only to the owner (600). The public key
/// file is given read/write permissions for the owner and read permissions for the
/// group (640). https://www.redhat.com/sysadmin/linux-file-permissions-explained
pub async fn gen_key_pair(
    num_bits: u32,
    private_key_file: &File,
    public_key_file: &File,
    overwrite: bool,
) -> Result<(), CryptErr> {
    // Generate the RSA key pair
    let rsa = Rsa::generate(num_bits).map_err(|e| {
        CryptErr::GenerateRSAKeyPairErr(GenerateRSAKeyPairErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // Extract and write the private key
    let private_key_pem = rsa.private_key_to_pem().map_err(|e| {
        CryptErr::ConvertPrivateKeyToPEMErr(ConvertPrivateKeyToPEMErr {
            source: e,
            trace: trace!(),
        })
    })?;
    private_key_file
        .write_bytes(&private_key_pem, overwrite, true)
        .await
        .map_err(|e| {
            CryptErr::FileSysErr(CryptFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
    // 600 gives the owner read/write permissions. Permissions to the group and others
    // are not granted.
    private_key_file.set_permissions(0o600).await.map_err(|e| {
        CryptErr::FileSysErr(CryptFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // Extract and write the public key
    let public_key_pem = rsa.public_key_to_pem().map_err(|e| {
        CryptErr::ConvertPrivateKeyToPEMErr(ConvertPrivateKeyToPEMErr {
            source: e,
            trace: trace!(),
        })
    })?;
    public_key_file
        .write_bytes(&public_key_pem, overwrite, true)
        .await
        .map_err(|e| {
            CryptErr::FileSysErr(CryptFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
    // 640 gives the owner read/write permissions, the group read permissions, and
    // nothing for other
    public_key_file.set_permissions(0o640).await.map_err(|e| {
        CryptErr::FileSysErr(CryptFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    Ok(())
}

/// Read an RSA private key from the specified file.
pub async fn read_private_key(private_key_file: &File) -> Result<Rsa<Private>, CryptErr> {
    // Ensure the file exists
    private_key_file.assert_exists().map_err(|e| {
        CryptErr::FileSysErr(CryptFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // Read the private key
    let private_key_pem = private_key_file.read_secret_bytes().await.map_err(|e| {
        CryptErr::FileSysErr(CryptFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;
    Rsa::private_key_from_pem(private_key_pem.expose_secret()).map_err(|e| {
        CryptErr::ReadKeyErr(ReadKeyErr {
            source: e,
            trace: trace!(),
        })
    })
}

/// Read an RSA public key from the specified file.
pub async fn read_public_key(public_key_file: &File) -> Result<Rsa<Public>, CryptErr> {
    public_key_file.assert_exists().map_err(|e| {
        CryptErr::FileSysErr(CryptFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // Read the public key
    let public_key_pem = public_key_file.read_secret_bytes().await.map_err(|e| {
        CryptErr::FileSysErr(CryptFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;
    Rsa::public_key_from_pem(public_key_pem.expose_secret()).map_err(|e| {
        CryptErr::ReadKeyErr(ReadKeyErr {
            source: e,
            trace: trace!(),
        })
    })
}

/// Create a signature from the provided data using the private key stored in the
/// specified file
pub async fn sign(private_key_file: &File, data: &[u8]) -> Result<Vec<u8>, CryptErr> {
    // Read the private key
    let rsa_private_key = read_private_key(private_key_file).await?;
    let private_key = PKey::from_rsa(rsa_private_key).map_err(|e| {
        CryptErr::RSAToPKeyErr(RSAToPKeyErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // Sign the data
    let mut signer = Signer::new(MessageDigest::sha256(), &private_key).map_err(|e| {
        CryptErr::SignDataErr(SignDataErr {
            source: e,
            trace: trace!(),
        })
    })?;
    signer.update(data).map_err(|e| {
        CryptErr::SignDataErr(SignDataErr {
            source: e,
            trace: trace!(),
        })
    })?;
    let signature = signer.sign_to_vec().map_err(|e| {
        CryptErr::SignDataErr(SignDataErr {
            source: e,
            trace: trace!(),
        })
    })?;
    Ok(signature)
}

/// Verify a signature using the public key stored in the specified file
pub async fn verify(
    public_key_file: &File,
    data: &[u8],
    signature: &[u8],
) -> Result<bool, CryptErr> {
    // Read the public key
    let rsa_public_key = read_public_key(public_key_file).await?;
    let public_key = PKey::from_rsa(rsa_public_key).map_err(|e| {
        CryptErr::RSAToPKeyErr(RSAToPKeyErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // Verify the signature
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).map_err(|e| {
        CryptErr::VerifyDataErr(VerifyDataErr {
            source: e,
            trace: trace!(),
        })
    })?;
    verifier.update(data).map_err(|e| {
        CryptErr::VerifyDataErr(VerifyDataErr {
            source: e,
            trace: trace!(),
        })
    })?;
    let is_valid = verifier.verify(signature).map_err(|e| {
        CryptErr::VerifyDataErr(VerifyDataErr {
            source: e,
            trace: trace!(),
        })
    })?;
    Ok(is_valid)
}
