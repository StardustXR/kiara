use glam::{vec2, Mat4, Vec2, Vec3, Vec3Swizzles};
use map_range::MapRange;
use mint::Vector2;
use stardust_xr_fusion::{
	core::values::ResourceID,
	drawable::{Lines, Model},
	fields::{CylinderShape, Field, Shape},
	input::{InputDataType, InputHandler, Pointer},
	items::panel::{PanelItem, PanelItemAspect, SurfaceId},
	node::NodeType,
	spatial::Transform,
};
use stardust_xr_molecules::{
	data::SimplePulseReceiver,
	input_action::{InputQueue, InputQueueable, SimpleAction},
	keyboard::KeyboardEvent,
	lines::{circle, LineExt},
};
use std::f32::consts::{FRAC_PI_2, PI};

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
	_field: Field,
	size_pixels: Vector2<u32>,
	input: InputQueue,
	hover_action: SimpleAction,
	click_action: SimpleAction,
	context_action: SimpleAction,
	_keyboard: SimplePulseReceiver<KeyboardEvent>,
	_lines: Lines,
}
impl Ring {
	pub fn new(panel_item: PanelItem) -> Self {
		let client = panel_item.node().client().unwrap();
		let _model = Model::create(
			client.get_root(),
			Transform::identity(),
			&ResourceID::new_namespaced("kiara", "ring"),
		)
		.unwrap();
		let ring = _model.part("Ring").unwrap();
		let field = Field::create(
			&_model,
			Transform::identity(),
			Shape::Cylinder(CylinderShape {
				length: HEIGHT_METERS,
				radius: RADIUS,
			}),
		)
		.unwrap();
		let input = InputHandler::create(&_model, Transform::identity(), &field)
			.unwrap()
			.queue()
			.unwrap();

		let panel_alias = panel_item.alias();
		let keyboard = SimplePulseReceiver::create(
			&_model,
			Transform::identity(),
			&field,
			move |_, keyboard_event: KeyboardEvent| {
				keyboard_event
					.send_to_panel(&panel_alias, SurfaceId::Toplevel(()))
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
			.apply_surface_material(SurfaceId::Toplevel(()), &ring)
			.unwrap();

		let circle = circle(128, 0.0, RADIUS + 0.01)
			.thickness(0.01)
			.transform(Mat4::from_rotation_x(FRAC_PI_2));
		let lines = Lines::create(
			&_model,
			Transform::identity(),
			&[
				circle.clone().transform(Mat4::from_translation(
					[0.0, HEIGHT_METERS / 2.0, 0.0].into(),
				)),
				circle.transform(Mat4::from_translation(
					[0.0, -HEIGHT_METERS / 2.0, 0.0].into(),
				)),
			],
		)
		.unwrap();
		Ring {
			panel_item,
			_model,
			_field: field,
			size_pixels,
			input,
			hover_action: Default::default(),
			click_action: Default::default(),
			context_action: Default::default(),
			_keyboard: keyboard,
			_lines: lines,
		}
	}
	pub fn update(&mut self) {
		self.hover_action
			.update(&self.input, &|data| match &data.input {
				InputDataType::Pointer(p) => map_pointer_screen_coords(p).is_some(),
				InputDataType::Hand(h) => map_point_screen_coords(h.palm.position.into()).is_some(),
				InputDataType::Tip(t) => map_point_screen_coords(t.origin.into()).is_some(),
			});
		self.click_action
			.update(&self.input, &|data| match data.input {
				InputDataType::Pointer(_) => {
					data.datamap.with_data(|r| r.idx("select").as_f32() > 0.0)
				}
				_ => false,
			});
		self.context_action
			.update(&self.input, &|data| match data.input {
				InputDataType::Pointer(_) => {
					data.datamap.with_data(|r| r.idx("context").as_f32() > 0.0)
				}
				_ => false,
			});

		if let Some(hover_actor) = self
			.hover_action
			.currently_acting()
			.difference(self.hover_action.started_acting())
			.reduce(|a, b| if a.distance > b.distance { b } else { a })
		{
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
					.pointer_motion(SurfaceId::Toplevel(()), pointer_pos)
					.unwrap();
				if self.click_action.started_acting().contains(hover_actor) {
					self.panel_item
						.pointer_button(
							SurfaceId::Toplevel(()),
							input_event_codes::BTN_LEFT!(),
							true,
						)
						.unwrap();
				}
				if self.click_action.stopped_acting().contains(hover_actor) {
					self.panel_item
						.pointer_button(
							SurfaceId::Toplevel(()),
							input_event_codes::BTN_LEFT!(),
							false,
						)
						.unwrap();
				}
				if self.context_action.started_acting().contains(hover_actor) {
					self.panel_item
						.pointer_button(
							SurfaceId::Toplevel(()),
							input_event_codes::BTN_RIGHT!(),
							true,
						)
						.unwrap();
				}
				if self.context_action.stopped_acting().contains(hover_actor) {
					self.panel_item
						.pointer_button(
							SurfaceId::Toplevel(()),
							input_event_codes::BTN_RIGHT!(),
							false,
						)
						.unwrap();
				}
			}
		}
	}
}
