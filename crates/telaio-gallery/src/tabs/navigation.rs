//! The "Navigation" tab: underline tabs, document tabs and tree rows.

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    DocumentTab, DocumentTabs, IconButton, SecondaryButton, TreeRow, UnderlineTabItem,
    UnderlineTabs,
};
use telaio_ui::metrics::RADIUS_LG;

use crate::gallery::{DemoTabs, Gallery, section, visible_nodes};

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_navigation_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Underline tabs", self.render_underline_tabs(cx)))
            .child(section("Document tabs", self.render_document_tabs(cx)))
            .child(section("Tree row", self.render_tree_rows(cx)))
    }

    /// Underline tabs: counts, a colored count, and a suffix.
    pub(crate) fn render_underline_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let names = ["Params", "Headers", "Body", "Tests", "Pre-request"];
        let tabs = names.into_iter().enumerate().fold(
            UnderlineTabs::new("gallery-underline-tabs"),
            |tabs, (index, name)| {
                let mut item = UnderlineTabItem::new(name)
                    .selected(self.underline_tab == index)
                    .on_click(Self::act(cx, move |this, _, cx| {
                        this.underline_tab = index;
                        this.record(format!("Tab {name}"), cx);
                    }));
                if name == "Headers" {
                    item = item.count("2");
                }
                if name == "Tests" {
                    item = item.count("3/3").count_color(palette.success);
                }
                tabs.item(item)
            },
        );
        tabs.suffix(
            div()
                .text_size(px(12.0))
                .text_color(palette.fg_subtle)
                .child("suffix"),
        )
    }

    /// Document tabs: a few tabs that fit, and twenty in a fixed width strip that overflows.
    pub(crate) fn render_document_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let caption = |text: &'static str| {
            div()
                .text_size(px(12.0))
                .text_color(palette.fg_muted)
                .child(text)
        };
        v_flex()
            .gap_3()
            .child(caption(
                "Fits: select, close, drag to reorder, middle click closes. \
                 The first two tabs have a prefix.",
            ))
            .child(
                self.document_tabs_demo(cx, "gallery-doc-tabs-few", &self.few_tabs, |this| {
                    &mut this.few_tabs
                }),
            )
            .child(caption(
                "Overflows (560 px wide): wheel, arrows and the tab list; \
                 adding or selecting a tab scrolls it into view.",
            ))
            .child(div().w(px(560.0)).child(self.document_tabs_demo(
                cx,
                "gallery-doc-tabs-many",
                &self.many_tabs,
                |this| &mut this.many_tabs,
            )))
    }

    /// One document tabs demo with its "toggle dirty" button. `state` is what `field` picks.
    fn document_tabs_demo(
        &self,
        cx: &mut Context<Self>,
        id: &'static str,
        state: &DemoTabs,
        field: fn(&mut Self) -> &mut DemoTabs,
    ) -> impl IntoElement {
        let palette = cx.palette();
        let bar = state
            .tabs
            .iter()
            .enumerate()
            .fold(DocumentTabs::new(id), |bar, (index, tab)| {
                let mut item = DocumentTab::new(tab.name.clone())
                    .selected(state.selected == index)
                    .dirty(tab.dirty)
                    .tooltip(format!("workspace/{}", tab.name))
                    .on_click(Self::act(cx, move |this, _, cx| {
                        field(this).selected = index;
                        cx.notify();
                    }))
                    .on_close(Self::act(cx, move |this, _, cx| {
                        field(this).close(index);
                        cx.notify();
                    }));
                if tab.prefixed {
                    let color = palette.success;
                    item = item.prefix(move || {
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(color)
                            .child("GET")
                    });
                }
                bar.item(item)
            });
        let bar = bar
            .on_reorder({
                let weak = cx.weak_entity();
                move |from, to, _, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        field(this).reorder(from, to);
                        cx.notify();
                    });
                }
            })
            .suffix(
                IconButton::new((id, 0usize), IconName::Plus)
                    .tooltip("New tab")
                    .on_click(Self::click(cx, move |this, _, cx| {
                        field(this).add();
                        cx.notify();
                    })),
            );
        v_flex().gap_2().child(bar).child(h_flex().child(
            SecondaryButton::new((id, 1usize), "Toggle dirty on the selected tab").on_click(
                Self::click(cx, move |this, _, cx| {
                    field(this).toggle_dirty();
                    cx.notify();
                }),
            ),
        ))
    }

    /// Tree rows: a small tree whose folders really expand and collapse, with a leading element
    /// on requests, a play button shown on hover, and a selected row.
    pub(crate) fn render_tree_rows(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let rows = visible_nodes(&self.tree).into_iter().map(|index| {
            let node = &self.tree[index];
            let mut row = TreeRow::new(("gallery-tree-row", index), node.label)
                .depth(node.depth)
                .selected(self.tree_selected == Some(index));
            if node.folder {
                row = row.folder(node.expanded);
            } else {
                row = row
                    .leading(
                        div()
                            .font_family(telaio_ui::fonts::MONO_FAMILY)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.success)
                            .child(if index % 2 == 0 { "GET" } else { "POST" }),
                    )
                    .trailing(
                        IconButton::new(("gallery-tree-play", index), IconName::Play).on_click(
                            Self::click(cx, move |this, _, cx| {
                                // Keep the row's own click from selecting it.
                                cx.stop_propagation();
                                let label = this.tree[index].label;
                                this.record(format!("Tree row: play {label}"), cx);
                            }),
                        ),
                    );
            }
            row.on_click(Self::click(cx, move |this, _, cx| {
                let node = &mut this.tree[index];
                if node.folder {
                    node.expanded = !node.expanded;
                    let (label, expanded) = (node.label, node.expanded);
                    let state = if expanded { "expanded" } else { "collapsed" };
                    this.record(format!("Tree row: {label} {state}"), cx);
                } else {
                    this.tree_selected = Some(index);
                    let label = node.label;
                    this.record(format!("Tree row: selected {label}"), cx);
                }
            }))
        });
        v_flex()
            .w(px(320.0))
            .p_2()
            .gap_px()
            .border_1()
            .border_color(palette.border)
            .rounded(px(RADIUS_LG))
            .bg(palette.surface)
            .children(rows)
    }
}
