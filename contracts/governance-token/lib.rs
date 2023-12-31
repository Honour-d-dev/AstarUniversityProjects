// Implement PSP2 + PSP22Metadata

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(
    PSP22,
   // PSP22Burnable,
    PSP22Mintable,
    PSP22Metadata,
   // PSP22Capped
)]
#[openbrush::contract]
pub mod my_psp22 {
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
        cap: Balance,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            initial_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
        ) -> Self {
            let mut _instance = Self::default();
            psp22::Internal::_mint_to(
                &mut _instance,
                Self::env().caller(),
                initial_supply,
            )
            .expect("Should mint");
            _instance.metadata.name.set(&name);
            _instance.metadata.symbol.set(&symbol);
            _instance.metadata.decimals.set(&decimal);
            _instance
        }

        #[ink(message)]
        pub fn cap(&self) -> Balance {
            self.cap
        }

        fn _init_cap(&mut self, cap: Balance) -> Result<(), PSP22Error> {
            if cap <= 0 {
                return Err(PSP22Error::Custom(String::from("Cap must be above 0")))
            }
            self.cap = cap;
            Ok(())
        }
    }
}
