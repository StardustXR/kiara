use std::f32::consts::PI;

use color::rgba_linear;
use glam::{vec2, Vec2, Vec3, Vec3Swizzles};
use map_range::MapRange;
use mint::Vector2;
use stardust_xr_fusion::{
	core::values::ResourceID,
	drawable::{Line, Lines, Model},
	fields::CylinderField,
	input::{InputDataType, InputHandler, Pointer},
	items::panel::{PanelItem, SurfaceID},
	node::NodeType,
	spatial::Transform,
	HandlerWrapper,
};
use stardust_xr_molecules::{
	data::SimplePulseReceiver,
	input_action::{BaseInputAction, InputActionHandler, SingleActorAction},
	keyboard::KeyboardEvent,
	lines::{circle, make_line_points},
};

const ANGLE: f32 = 360.0;
const RADIUS: f32 = 2.0;
const HEIGHT_METERS: f32 = 1.0;
const HEIGHT_PIXELS: u32 = 1080;

fn ray_circle_closest_intersection(
	ray_origin: Vec2,
	ray_direction: Vec2,
	circle_radius: f32,
) -> Option<f32> {
	let a = ray_direction.x * ray_direction.x + ray_direction.y * ray_direction.y;
	let b = 2.0 * (ray_direction.x * ray_origin.x + ray_direction.y * ray_origin.y);
	let c =
		ray_origin.x * ray_origin.x + ray_origin.y * ray_origin.y - circle_radius * circle_radius;

	let disc = b * b - 4.0 * a * c;
	if disc < 0.0 {
		return None;
	}

	let t1 = (-b - disc.sqrt()) / (2.0 * a);
	if t1 >= 0.0 {
		return Some(t1);
	}

	let t2 = (-b + disc.sqrt()) / (2.0 * a);
	if t2 >= 0.0 {
		return Some(t2);
	}

	None
}
fn map_pointer_screen_coords(pointer: &Pointer) -> Option<Vec2> {
	let origin: Vec3 = pointer.origin.into();
	let direction: Vec3 = pointer.direction().into();
	let t_2d = ray_circle_closest_intersection(origin.xz(), direction.xz().normalize(), RADIUS)?;
	let xz_length = direction.xz().length();
	let t = t_2d / xz_length;
	let intersection_point = origin + (direction * t);
	map_point_screen_coords(intersection_point)
}
fn map_point_screen_coords(point: Vec3) -> Option<Vec2> {
	if point.y.abs() > HEIGHT_METERS / 2.0 {
		return None;
	}

	let x = point.x.atan2(-point.z).map_range(-PI..PI, 0.0..1.0);
	let y = point
		.y
		.map_range(HEIGHT_METERS / 2.0..-HEIGHT_METERS / 2.0, 0.0..1.0);
	Some(vec2(x, y))
}

pub struct Ring {
	panel_item: PanelItem,
	_model: Model,
	_field: CylinderField,
	size_pixels: Vector2<u32>,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<()>>,
	hover_action: SingleActorAction<()>,
	click_action: BaseInputAction<()>,
	context_action: BaseInputAction<()>,
	_keyboard: SimplePulseReceiver<KeyboardEvent>,
	_lines: Lines,
}
impl Ring {
	pub fn new(panel_item: PanelItem) -> Self {
		let client = panel_item.node().client().unwrap();
		let model = Model::create(
			client.get_root(),
			Transform::identity(),
			&ResourceID::new_namespaced("kiara", "ring"),
		)
		.unwrap();
		let ring = model.model_part("Ring").unwrap();
		let field =
			CylinderField::create(&model, Transform::identity(), RADIUS, HEIGHT_METERS).unwrap();
		let input_handler = InputActionHandler::wrap(
			InputHandler::create(&model, Transform::identity(), &field).unwrap(),
			(),
		)
		.unwrap();
		let hover_action = SingleActorAction::new(
			false,
			|data, _: &()| match &data.input {
				InputDataType::Pointer(p) => map_pointer_screen_coords(&p).is_some(),
				InputDataType::Hand(h) => map_point_screen_coords(h.palm.position.into()).is_some(),
				InputDataType::Tip(t) => map_point_screen_coords(t.origin.into()).is_some(),
			},
			true,
		);
		let click_action = BaseInputAction::new(false, |data, _: &()| match data.input {
			InputDataType::Pointer(_) => data.datamap.with_data(|r| r.idx("select").as_f32() > 0.0),
			_ => false,
		});
		let context_action = BaseInputAction::new(false, |data, _: &()| match data.input {
			InputDataType::Pointer(_) => {
				data.datamap.with_data(|r| r.idx("context").as_f32() > 0.0)
			}
			_ => false,
		});

		let panel_alias = panel_item.alias();
		let keyboard = SimplePulseReceiver::create(
			&model,
			Transform::identity(),
			&field,
			move |_, keyboard_event: KeyboardEvent| {
				keyboard_event
					.send_to_panel(&panel_alias, &SurfaceID::Toplevel)
					.unwrap();
			},
		)
		.unwrap();

		let arc_length = ANGLE.to_radians() * RADIUS;
		let aspect_ratio = arc_length / HEIGHT_METERS;
		let width_pixels = aspect_ratio * (HEIGHT_PIXELS as f32);
		let size_pixels = [width_pixels as u32, HEIGHT_PIXELS].into();
		panel_item.set_toplevel_size(size_pixels).unwrap();
		panel_item
			.apply_surface_material(&SurfaceID::Toplevel, &ring)
			.unwrap();

		let circle = circle(128, 0.0, RADIUS + 0.01);
		let lines = Lines::create(
			&model,
			Transform::identity(),
			&[
				Line {
					points: make_line_points(
						circle
							.iter()
							.map(|p| [p.x, HEIGHT_METERS / 2.0, p.y].into())
							.collect(),
						0.01,
						rgba_linear!(1.0, 1.0, 1.0, 1.0),
					),
					cyclic: true,
				},
				Line {
					points: make_line_points(
						circle
							.iter()
							.map(|p| [p.x, -HEIGHT_METERS / 2.0, p.y].into())
							.collect(),
						0.01,
						rgba_linear!(1.0, 1.0, 1.0, 1.0),
					),
					cyclic: true,
				},
			],
		)
		.unwrap();
		Ring {
			panel_item,
			_model: model,
			_field: field,
			size_pixels,
			input_handler,
			hover_action,
			click_action,
			context_action,
			_keyboard: keyboard,
			_lines: lines,
		}
	}
	pub fn update(&mut self) {
		self.input_handler.lock_wrapped().update_actions([
			self.hover_action.base_mut(),
			&mut self.click_action,
			&mut self.context_action,
		]);
		self.hover_action.update(None);
		if let Some(hover_actor) = self.hover_action.actor() {
			if let Some(pointer_pos) = match &hover_actor.input {
				InputDataType::Pointer(p) => map_pointer_screen_coords(p),
				InputDataType::Hand(_) => None,
				InputDataType::Tip(_) => None,
			} {
				let pointer_pos = vec2(
					pointer_pos.x * self.size_pixels.x as f32,
					pointer_pos.y * self.size_pixels.y as f32,
				);
				self.panel_item
					.pointer_motion(&SurfaceID::Toplevel, pointer_pos)
					.unwrap();
				if self.click_action.started_acting.contains(hover_actor) {
					self.panel_item
						.pointer_button(&SurfaceID::Toplevel, input_event_codes::BTN_LEFT!(), true)
						.unwrap();
				}
				if self.click_action.stopped_acting.contains(hover_actor) {
					self.panel_item
						.pointer_button(&SurfaceID::Toplevel, input_event_codes::BTN_LEFT!(), false)
						.unwrap();
				}
				if self.context_action.started_acting.contains(hover_actor) {
					self.panel_item
						.pointer_button(&SurfaceID::Toplevel, input_event_codes::BTN_RIGHT!(), true)
						.unwrap();
				}
				if self.context_action.stopped_acting.contains(hover_actor) {
					self.panel_item
						.pointer_button(
							&SurfaceID::Toplevel,
							input_event_codes::BTN_RIGHT!(),
							false,
						)
						.unwrap();
				}
			}
		}
	}
}
