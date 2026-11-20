//! Interactive 3D Orbit Viewer for build volumes, 3D G-code toolpaths,
//! and ASCII/binary STL parts placed on the print bed.

use egui::{Color32, Pos2, Rect, Stroke};
use makercraft_core::{MachineProfile, ParsedToolpath, StlMesh};

pub struct Viewer3dState {
    pub yaw_rad: f32,
    pub pitch_rad: f32,
    pub zoom: f32,
}

impl Default for Viewer3dState {
    fn default() -> Self {
        Self {
            yaw_rad: 0.65,
            pitch_rad: 0.52,
            zoom: 1.0,
        }
    }
}

pub struct Viewer3dOpts {
    pub show_grid: bool,
    pub show_rapids: bool,
    pub z_filter_enabled: bool,
    pub z_filter_max: f64,
}

impl Viewer3dState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        profile: &MachineProfile,
        toolpath: Option<&ParsedToolpath>,
        stl: Option<&StlMesh>,
        opts: Viewer3dOpts,
    ) {
        let resp = ui.interact(
            rect,
            ui.scope_id().with("viewer_3d"),
            egui::Sense::click_and_drag(),
        );
        if resp.dragged() {
            let d = resp.drag_delta();
            self.yaw_rad += d.x * 0.01;
            self.pitch_rad = (self.pitch_rad + d.y * 0.01).clamp(-1.45, 1.45);
        }
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if resp.hovered() && wheel != 0.0 {
            let factor = (1.0 + wheel * 0.0015).clamp(0.8, 1.25);
            self.zoom = (self.zoom * factor).clamp(0.25, 6.0);
        }

        let p = ui.painter_at(rect);
        p.rect_filled(rect, 6.0, Color32::from_rgb(10, 14, 24));
        p.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.5, Color32::from_rgb(48, 72, 112)),
            egui::StrokeKind::Inside,
        );

        let bx = profile.bed_x_mm as f32;
        let by = profile.bed_y_mm as f32;
        let bz = profile.max_z_mm.max(40.0) as f32;
        let cx = bx * 0.5;
        let cy = by * 0.5;
        let cz = bz * 0.28;

        let max_dim = bx.max(by).max(bz * 0.6).max(50.0);
        let scale = (rect.width().min(rect.height()) * 0.62 / max_dim) * self.zoom;
        let (sy, cy_ang) = self.yaw_rad.sin_cos();
        let (sp, cp) = self.pitch_rad.sin_cos();

        // Project 3D world (X right, Y depth, Z up) into (screen Pos2, depth_cam)
        let project = |wx: f32, wy: f32, wz: f32| -> (Pos2, f32) {
            let dx = wx - cx;
            let dy = wy - cy;
            let dz = wz - cz;
            let rx = dx * cy_ang - dy * sy;
            let ry = dx * sy + dy * cy_ang;
            let rz = dz;
            let py = ry * cp - rz * sp;
            let pz = ry * sp + rz * cp;
            (
                Pos2::new(rect.center().x + rx * scale, rect.center().y - pz * scale),
                py,
            )
        };

        // 1. Optional 20mm bed grid at Z = 0
        if opts.show_grid {
            let g_stroke = Stroke::new(0.7, Color32::from_rgb(24, 34, 52));
            let mut gx = 0.0;
            while gx <= bx {
                let (a, _) = project(gx, 0.0, 0.0);
                let (b, _) = project(gx, by, 0.0);
                p.line_segment([a, b], g_stroke);
                gx += 20.0;
            }
            let mut gy = 0.0;
            while gy <= by {
                let (a, _) = project(0.0, gy, 0.0);
                let (b, _) = project(bx, gy, 0.0);
                p.line_segment([a, b], g_stroke);
                gy += 20.0;
            }
        }

        // 2. Build-volume wireframe cube
        let corners = [
            project(0.0, 0.0, 0.0).0,
            project(bx, 0.0, 0.0).0,
            project(bx, by, 0.0).0,
            project(0.0, by, 0.0).0,
            project(0.0, 0.0, bz).0,
            project(bx, 0.0, bz).0,
            project(bx, by, bz).0,
            project(0.0, by, bz).0,
        ];
        let bed_stroke = Stroke::new(1.4, Color32::from_rgb(180, 145, 70));
        let top_stroke = Stroke::new(0.9, Color32::from_rgb(56, 76, 108));
        for (i, j) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
            p.line_segment([corners[i], corners[j]], bed_stroke);
        }
        for (i, j) in [
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ] {
            p.line_segment([corners[i], corners[j]], top_stroke);
        }

        // XYZ origin triad
        let orig = project(0.0, 0.0, 0.0).0;
        p.line_segment(
            [orig, project(28.0, 0.0, 0.0).0],
            Stroke::new(2.0, Color32::from_rgb(255, 100, 100)),
        );
        p.line_segment(
            [orig, project(0.0, 28.0, 0.0).0],
            Stroke::new(2.0, Color32::from_rgb(100, 240, 140)),
        );
        p.line_segment(
            [orig, project(0.0, 0.0, 28.0).0],
            Stroke::new(2.0, Color32::from_rgb(110, 180, 255)),
        );

        // 3. Optional STL Part Mesh (centered on bed if outside, or placed at Z>=0)
        if let Some(mesh) = stl {
            let mx = (mesh.min[0] + mesh.max[0]) * 0.5;
            let my = (mesh.min[1] + mesh.max[1]) * 0.5;
            let off_x = cx - mx;
            let off_y = cy - my;
            let off_z = -mesh.min[2];

            // Cap drawn triangles to keep immediate-mode FPS smooth for huge meshes
            let step = (mesh.triangles.len() / 2500).max(1);
            let mut sorted: Vec<(f32, [Pos2; 3], Color32)> = Vec::new();
            for tri in mesh.triangles.iter().step_by(step) {
                let p0 = project(
                    tri.verts[0][0] + off_x,
                    tri.verts[0][1] + off_y,
                    tri.verts[0][2] + off_z,
                );
                let p1 = project(
                    tri.verts[1][0] + off_x,
                    tri.verts[1][1] + off_y,
                    tri.verts[1][2] + off_z,
                );
                let p2 = project(
                    tri.verts[2][0] + off_x,
                    tri.verts[2][1] + off_y,
                    tri.verts[2][2] + off_z,
                );
                let depth = (p0.1 + p1.1 + p2.1) / 3.0;
                // Simple Lambertian shading from top-right light
                let light = (tri.normal[0] * 0.35 + tri.normal[1] * 0.45 + tri.normal[2] * 0.82)
                    .abs()
                    .clamp(0.25, 1.0);
                let r = (75.0 * light) as u8;
                let g = (195.0 * light) as u8;
                let b = (185.0 * light) as u8;
                sorted.push((depth, [p0.0, p1.0, p2.0], Color32::from_rgb(r, g, b)));
            }
            sorted.sort_by(|a, b| b.0.total_cmp(&a.0));
            let edge = Stroke::new(0.6, Color32::from_rgb(160, 245, 225));
            for (_, pts, fill) in sorted {
                p.add(egui::Shape::convex_polygon(pts.to_vec(), fill, edge));
            }
        }

        // 4. 3D Toolpath segments
        if let Some(path) = toolpath {
            let z_span = (path.max_z_mm - path.min_z_mm).max(1.0);
            for seg in &path.segments {
                if seg.rapid && !opts.show_rapids {
                    continue;
                }
                if opts.z_filter_enabled && seg.to_z_mm > opts.z_filter_max {
                    continue;
                }
                let a = project(
                    seg.from_x_mm as f32,
                    seg.from_y_mm as f32,
                    seg.from_z_mm as f32,
                )
                .0;
                let b = project(seg.to_x_mm as f32, seg.to_y_mm as f32, seg.to_z_mm as f32).0;
                let stroke = if seg.rapid {
                    Stroke::new(1.2, Color32::from_rgb(255, 125, 125))
                } else {
                    let t = ((seg.to_z_mm - path.min_z_mm) / z_span).clamp(0.0, 1.0) as f32;
                    let col = Color32::from_rgb(
                        (110.0 + t * 145.0) as u8,
                        (240.0 - t * 35.0) as u8,
                        (205.0 - t * 100.0) as u8,
                    );
                    Stroke::new(2.2, col)
                };
                p.line_segment([a, b], stroke);
            }
        }

        p.text(
            Pos2::new(rect.left() + 10.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "3D ORBIT VIEW: drag to rotate | wheel to zoom",
            egui::FontId::monospace(10.5),
            Color32::from_rgb(186, 206, 232),
        );
    }
}
