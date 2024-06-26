/*
 * Copyright (c) 2023 Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Server.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use mail_auth::{
    arc::ArcSet, dkim::Signature, dmarc::Policy, ArcOutput, AuthenticatedMessage,
    AuthenticationResults, DkimResult, DmarcResult, IprevResult, SpfResult,
};

use crate::config::{ArcSealer, DkimSigner};

pub mod auth;
pub mod data;
pub mod ehlo;
pub mod mail;
pub mod milter;
pub mod rcpt;
pub mod session;
pub mod spawn;
pub mod vrfy;

impl ArcSealer {
    pub fn seal<'x>(
        &self,
        message: &'x AuthenticatedMessage,
        results: &'x AuthenticationResults,
        arc_output: &'x ArcOutput,
    ) -> mail_auth::Result<ArcSet<'x>> {
        match self {
            ArcSealer::RsaSha256(sealer) => sealer.seal(message, results, arc_output),
            ArcSealer::Ed25519Sha256(sealer) => sealer.seal(message, results, arc_output),
        }
    }
}

impl DkimSigner {
    pub fn sign(&self, message: &[u8]) -> mail_auth::Result<Signature> {
        match self {
            DkimSigner::RsaSha256(signer) => signer.sign(message),
            DkimSigner::Ed25519Sha256(signer) => signer.sign(message),
        }
    }
    pub fn sign_chained(&self, message: &[&[u8]]) -> mail_auth::Result<Signature> {
        match self {
            DkimSigner::RsaSha256(signer) => signer.sign_chained(message.iter().copied()),
            DkimSigner::Ed25519Sha256(signer) => signer.sign_chained(message.iter().copied()),
        }
    }
}

pub trait AuthResult {
    fn as_str(&self) -> &'static str;
}

impl AuthResult for SpfResult {
    fn as_str(&self) -> &'static str {
        match self {
            SpfResult::Pass => "pass",
            SpfResult::Fail => "fail",
            SpfResult::SoftFail => "softfail",
            SpfResult::Neutral => "neutral",
            SpfResult::None => "none",
            SpfResult::TempError => "temperror",
            SpfResult::PermError => "permerror",
        }
    }
}

impl AuthResult for IprevResult {
    fn as_str(&self) -> &'static str {
        match self {
            IprevResult::Pass => "pass",
            IprevResult::Fail(_) => "fail",
            IprevResult::TempError(_) => "temperror",
            IprevResult::PermError(_) => "permerror",
            IprevResult::None => "none",
        }
    }
}

impl AuthResult for DkimResult {
    fn as_str(&self) -> &'static str {
        match self {
            DkimResult::Pass => "pass",
            DkimResult::None => "none",
            DkimResult::Neutral(_) => "neutral",
            DkimResult::Fail(_) => "fail",
            DkimResult::PermError(_) => "permerror",
            DkimResult::TempError(_) => "temperror",
        }
    }
}

impl AuthResult for DmarcResult {
    fn as_str(&self) -> &'static str {
        match self {
            DmarcResult::Pass => "pass",
            DmarcResult::Fail(_) => "fail",
            DmarcResult::TempError(_) => "temperror",
            DmarcResult::PermError(_) => "permerror",
            DmarcResult::None => "none",
        }
    }
}

impl AuthResult for Policy {
    fn as_str(&self) -> &'static str {
        match self {
            Policy::Reject => "reject",
            Policy::Quarantine => "quarantine",
            Policy::None | Policy::Unspecified => "none",
        }
    }
}
