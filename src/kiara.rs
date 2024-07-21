use crate::ring::Ring;
use stardust_xr_fusion::{
	fields::Field,
	items::{
		panel::{
			PanelItem, PanelItemAcceptor, PanelItemAcceptorHandler, PanelItemInitData,
			PanelItemUiHandler,
		},
		ItemAcceptorHandler, ItemUiHandler,
	},
	node::NodeType,
	root::{ClientState, FrameInfo, RootHandler},
};
use std::sync::OnceLock;

#[derive(Default)]
pub struct Kiara {
	ring: OnceLock<(u64, Ring)>,
}
impl Kiara {
	fn add_item(&mut self, uid: u64, item: PanelItem, _init_data: PanelItemInitData) {
		// dbg!(init_data);
		self.ring.get_or_init(|| {
			let ring = Ring::new(item);
			(uid, ring)
		});
	}
	fn remove_item(&mut self, uid: u64) {
		if let Some((this_uid, _)) = self.ring.get() {
			if *this_uid == uid {
				self.ring.take();
			}
		}
	}
}
impl PanelItemUiHandler for Kiara {
	fn create_item(&mut self, item: PanelItem, init_data: PanelItemInitData) {
		self.add_item(item.node().get_id().unwrap(), item, init_data);
	}
	fn create_acceptor(&mut self, _acceptor: PanelItemAcceptor, _acceptor_field: Field) {}
}
impl ItemUiHandler for Kiara {
	fn capture_item(&mut self, _item_id: u64, _acceptor_id: u64) {}
	fn release_item(&mut self, _item_id: u64, _acceptor_id: u64) {}

	fn destroy_item(&mut self, uid: u64) {
		self.remove_item(uid);
	}
	fn destroy_acceptor(&mut self, _uid: u64) {}
}
impl PanelItemAcceptorHandler for Kiara {
	fn capture_item(&mut self, item: PanelItem, initial_data: PanelItemInitData) {
		self.add_item(item.node().get_id().unwrap(), item, initial_data);
	}
}
impl ItemAcceptorHandler for Kiara {
	fn release_item(&mut self, uid: u64) {
		self.remove_item(uid);
	}
}
impl RootHandler for Kiara {
	fn frame(&mut self, _info: FrameInfo) {
		if let Some((_, ring)) = self.ring.get_mut() {
			ring.update();
		}
	}

	fn save_state(&mut self) -> color_eyre::eyre::Result<ClientState> {
		Ok(ClientState::default())
	}
}
