use std::sync::OnceLock;

use crate::ring::Ring;
use stardust_xr_fusion::{
	client::{ClientState, FrameInfo, RootHandler},
	items::{
		panel::{PanelItem, PanelItemInitData},
		ItemUIHandler,
	},
};

pub struct Kiara {
	ring: OnceLock<(String, Ring)>,
}
impl Kiara {
	pub fn new() -> Self {
		Kiara {
			ring: OnceLock::new(),
		}
	}

	fn add_item(&mut self, uid: String, item: PanelItem, _init_data: PanelItemInitData) {
		// dbg!(init_data);
		self.ring.get_or_init(|| {
			let ring = Ring::new(item);
			(uid, ring)
		});
	}
	fn remove_item(&mut self, uid: &str) {
		if let Some((this_uid, _)) = self.ring.get() {
			if this_uid == uid {
				self.ring.take();
			}
		}
	}
}
impl ItemUIHandler<PanelItem> for Kiara {
	fn item_created(&mut self, uid: String, item: PanelItem, init_data: PanelItemInitData) {
		self.add_item(uid, item, init_data);
	}
	fn item_destroyed(&mut self, uid: String) {
		self.remove_item(&uid);
	}
}
// impl ItemAcceptorHandler<PanelItem> for Flatland {
// 	fn captured(&mut self, uid: String, item: PanelItem, init_data: PanelItemInitData) {
// 		self.add_item(uid, item, init_data);
// 	}
// 	fn released(&mut self, uid: String) {
// 		self.remove_item(uid);
// 	}
// }
impl RootHandler for Kiara {
	fn frame(&mut self, _info: FrameInfo) {
		if let Some((_, ring)) = self.ring.get_mut() {
			ring.update();
		}
	}

	fn save_state(&mut self) -> ClientState {
		ClientState::default()
	}
}
