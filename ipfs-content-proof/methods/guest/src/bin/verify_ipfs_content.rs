// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]
use ipfs_core::IpfsProof;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    // Decode the verifying key, message, and signature from the inputs.
    //let proof_slice = env::read_slice(slice)
    let ipfs_proof_request: IpfsProof = env::read();
    

    // Verify the signature, panicking if verification fails.
    let res = ipfs_proof_request.calculate_proof();

    // Commit to the journal the verifying key and message that was signed.
    env::commit(&res.0);
    env::commit(&res.1);
    println!("Finished in {} cycles", env::get_cycle_count());
}