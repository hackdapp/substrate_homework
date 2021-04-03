#![cfg_attr(not(feature = "std"), no_std)]

// 1. 导入
use frame_support::{ ensure, decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

// 2. Configuration
pub trait Config: frame_system::Config {
  type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// 3. 存储
decl_storage! {
  trait Store for Module<T: Config> as TemplateModule {
    /// The storage item for our proofs.
    /// 它将证明映射到提出声明的用户以及声明的时间。
    Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
  }
}

// 4. 事件
decl_event! {
  pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
    /// Event emitted when a proof has been claimed. [who, claim]
    ClaimCreated(AccountId, Vec<u8>),
    /// Event emitted when a claim is revoked by the owner. [who, claim]
    ClaimRevoked(AccountId, Vec<u8>),
    /// Event emitted when a proof has been transferred. [reciver, claim]
    ClaimTransferred(AccountId, Vec<u8>),
  }
 }

// 5. 错误
decl_error! {
  pub enum Error for Module<T: Config> {
    /// The proof has already been claimed.
    ProofAlreadyClaimed,
    /// 该证明不存在，因此它不能被撤销
    NoSuchProof,
    /// 该证明已经被另一个账号声明，因此它不能被撤销
    NotProofOwner,
    OwnedClaimAlready,
  }
}

// 6. 可调用函数
decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {
    // Errors must be initialized if they are used by the pallet.
    type Error = Error<T>;

    // 事件必须被初始化，如果它们被模块所使用。
    fn deposit_event() = default;

    /// 允许用户队未声明的证明拥有所有权
    #[weight = 10_000]
    fn create_claim(origin, proof: Vec<u8>) {
        // 检查 extrinsic 是否签名并获得签名者
        // 如果 extrinsic 未签名，此函数将返回一个错误。
        // https://substrate.dev/docs/en/knowledgebase/runtime/origin
        let sender = ensure_signed(origin)?;

        // 校验指定的证明是否被声明
        ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

        // 从 FRAME 系统模块中获取区块号.
        let current_block = <frame_system::Module<T>>::block_number();

        // 存储证明：发送人与区块号
        Proofs::<T>::insert(&proof, (&sender, current_block));

        // 声明创建后，发送事件
        Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
    }

    /// 允许证明所有者撤回声明
    #[weight = 10_000]
    fn revoke_claim(origin, proof: Vec<u8>) {
        //  检查 extrinsic 是否签名并获得签名者
        // 如果 extrinsic 未签名，此函数将返回一个错误。
        // https://substrate.dev/docs/en/knowledgebase/runtime/origin
        let sender = ensure_signed(origin)?;

        // 校验指定的证明是否被声明
        ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

        // 获取声明的所有者
        let (owner, _) = Proofs::<T>::get(&proof);

        // 验证当前的调用者是证声明的所有者
        ensure!(sender == owner, Error::<T>::NotProofOwner);

        // 从存储中移除声明
        Proofs::<T>::remove(&proof);

        // 声明抹掉后，发送事件
        Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
    }

    #[weight = 10_000]
    fn trans_claim(origin, proof: Vec<u8>) -> dispatch::DispatchResult{
      let receiver = ensure_signed(origin)?;

      ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

      let (owner, _) = Proofs::<T>::get(&proof);
      ensure!(receiver != owner, Error::<T>::OwnedClaimAlready);

      let current_block = <frame_system::Module<T>>::block_number();
      Proofs::<T>::remove(&proof);
      Proofs::<T>::insert(&proof, (&receiver, current_block));

      Self::deposit_event(RawEvent::ClaimTransferred(receiver, proof));
      Ok(())
    }
  }
}
