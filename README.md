
# Selective Disclosure Trie

> "Selective Disclosure Trie" is a solution proposal that uses a data structure called a trie, to enable selective disclosure of fields from claims for integrity and privacy enhancement. It also aims to provide proof of not existence.

> Inspired by Merkle Patricia Trie 

## Example

```rust
 let personal = SdtNode::new()
    .add_str_value("name", "Adem")
    .add_str_value("surname", "Çağlın")
    .add_bool_value("age_over_18", false)
    .build();
 let assertion_method = SdtNode::new().add_str_value("key_1", "0x12").build();
 let keys = SdtNode::new().add_node("assertion_method", assertion_method).build();
 let root = SdtNode::new()
    .add_node("personal", personal)
    .add_node("keys", keys)
    .build();

// Create new trie with subject id and inception claims

let mut sdt = Sdt::new("did:p2p:123456", root).build();

// Get the root proof and anchor it in a public source
let proof = sdt.gen_proof()?;

// After a while mutate a claim then  

let mutation = SdtNode::new()
    .add_str_value("age_over_18", true)
    .build();

sdt.mutate(SdtNode::new().add_node("personal", mutation));

let proof = sdt.gen_proof()?; //Get  proof of mutation and change it in public source

// Query for specific claim 

let query = "
{
    personal {
        age_over_18
    }
}
";

// Now sdt contains only required to prove age_over_18 claim
let selected_sdt = sdt.select(query)?;
 
// You can verify proof's validity
let is_valid = selected_sdt.verify(proof)?;

```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.