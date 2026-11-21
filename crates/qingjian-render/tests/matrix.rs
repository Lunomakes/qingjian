//! 横排展开成矩阵：候选按 `columns` 排成行列，末行的空位不画、不能点；各内置主题都要排得出来。

mod scenes;

use qingjian_render::{FontLibrary, HitRegion, HitTarget, Layout, Renderer, Theme};

fn cell(hits: &[HitRegion], index: usize) -> Option<HitRegion> {
    hits.iter()
        .find(|region| region.target == HitTarget::Candidate(index))
        .copied()
}

#[test]
fn candidates_are_laid_out_in_rows_and_columns() {
    let Ok(library) = FontLibrary::system("zh-CN") else {
        eprintln!("没有系统字体，跳过");
        return;
    };
    let mut renderer = Renderer::new(library);
    let frame = scenes::matrix();
    let columns = frame.columns;
    for theme in Theme::builtins() {
        renderer.forget();
        let rendered = renderer
            .render(&frame, Layout::Horizontal, theme, 2.0)
            .unwrap();
        let hits = &rendered.hits;
        let what = theme.id();
        let at = |index| cell(hits, index).unwrap_or_else(|| panic!("{what}: 没有第 {index} 格"));
        for index in 0..13 {
            let (row, column) = (index / columns, index % columns);
            let here = at(index);
            if column > 0 {
                assert!(
                    at(index - 1).x < here.x,
                    "{what}: 第 {index} 格没排在左边那格右侧"
                );
            }
            if row > 0 {
                let above = at(index - columns);
                assert!(
                    above.y + above.height <= here.y + 0.5,
                    "{what}: 第 {index} 格没排在上一行下面"
                );
                assert!(
                    (above.x - here.x).abs() < 0.5,
                    "{what}: 第 {index} 格与上一行同列没对齐"
                );
            }
        }
        assert!(
            cell(hits, 13).is_none() && cell(hits, 14).is_none(),
            "{what}: 空位不该能点"
        );
    }
}

#[test]
fn single_row_is_unchanged_when_not_expanded() {
    let Ok(library) = FontLibrary::system("zh-CN") else {
        return;
    };
    let mut renderer = Renderer::new(library);
    let mut frame = scenes::matrix();
    frame.rows.truncate(5);
    frame.columns = 0;
    frame.column_ems.clear();
    frame.highlighted = Some(1);
    let rendered = renderer
        .render(&frame, Layout::Horizontal, &Theme::light(), 2.0)
        .unwrap();
    let first = cell(&rendered.hits, 0).unwrap();
    let last = cell(&rendered.hits, 4).unwrap();
    assert!((first.y - last.y).abs() < 0.5 && first.x < last.x);
}

#[test]
fn column_floor_makes_single_row_as_wide_as_the_grid_row() {
    let Ok(library) = FontLibrary::system("zh-CN") else {
        return;
    };
    let mut renderer = Renderer::new(library);
    let mut frame = scenes::matrix();
    let columns = frame.columns;
    for (i, row) in frame.rows.iter_mut().enumerate() {
        row.annotation.clear();
        row.index = (i % columns + 1).to_string();
    }
    frame.column_floor = true;
    frame.column_ems = vec![3.0; columns];
    let mut row = frame.clone();
    row.rows.truncate(columns);
    row.columns = 0;
    let mut width = |frame: &_| {
        let rendered = renderer
            .render(frame, Layout::Horizontal, &Theme::light(), 2.0)
            .unwrap();
        let (first, last) = (
            cell(&rendered.hits, 0).unwrap(),
            cell(&rendered.hits, columns - 1).unwrap(),
        );
        last.x + last.width - first.x
    };
    let (single, grid) = (width(&row), width(&frame));
    assert!((single - grid).abs() < 1.0, "{single} vs {grid}");
}
