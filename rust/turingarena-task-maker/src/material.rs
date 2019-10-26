extern crate mime_guess;

use std::convert::TryInto;
use std::path::{Path, PathBuf};
use task_maker_format::ioi;
use turingarena::content::*;
use turingarena::evaluation::record::*;
use turingarena::feedback::{table::*, *};
use turingarena::problem::material::*;
use turingarena::score::*;
use turingarena::submission::form::*;

fn subtasks_of(task: &ioi::Task) -> Vec<&ioi::SubtaskInfo> {
    let mut subtasks: Vec<_> = task.subtasks.values().collect();
    subtasks.sort_by(|a, b| a.id.cmp(&b.id));
    subtasks
}

fn testcases_of(subtask: &ioi::SubtaskInfo) -> Vec<&ioi::TestcaseInfo> {
    let mut testcases: Vec<_> = subtask.testcases.values().collect();
    testcases.sort_by(|a, b| a.id.cmp(&b.id));
    testcases
}

fn submission_form() -> Form {
    Form {
        fields: vec![Field {
            id: FieldId("solution".into()),
            title: vec![TextVariant {
                attributes: vec![],
                value: "Solution".into(),
            }],
            types: vec![FileType {
                id: FileTypeId("cpp".into()),
                title: vec![TextVariant {
                    attributes: vec![],
                    value: "C++".into(),
                }],
                extensions: vec![
                    FileTypeExtension(".cpp".into()),
                    FileTypeExtension(".cc".into()),
                ],
                primary_extension: FileTypeExtension(".cpp".into()),
            }],
        }],
    }
}

fn scorable_of(subtask: &ioi::SubtaskInfo) -> Scorable {
    Scorable {
        name: ScorableName(format!("subtask.{}", subtask.id)),
        title: vec![TextVariant {
            attributes: vec![],
            value: format!("Subtask {}", subtask.id),
        }],
        range: Range {
            // TODO: assuming IOI-like tasks have integer scores
            precision: 0,
            max: Score(subtask.max_score),
        },
    }
}

fn cols() -> Vec<Col> {
    vec![
        Col {
            title: vec![TextVariant {
                attributes: vec![],
                value: format!("Case"),
            }],
            content: ColContent::RowNumber(RowNumberColContent {}),
        },
        Col {
            title: vec![TextVariant {
                attributes: vec![],
                value: format!("Score"),
            }],
            content: ColContent::Score(ScoreColContent {
                range: Range {
                    // FIXME: assuming per-test-case score has fixed precision
                    precision: 2,
                    max: Score(1.),
                },
            }),
        },
    ]
}

fn caption() -> Text {
    vec![TextVariant {
        attributes: vec![],
        value: format!("Test case results"),
    }]
}

fn row_group_of(subtask: &ioi::SubtaskInfo) -> RowGroup {
    RowGroup {
        title: vec![TextVariant {
            attributes: vec![],
            value: format!("Subtask {}", subtask.id),
        }],
        rows: testcases_of(subtask).into_iter().map(row_of).collect(),
    }
}

fn row_of(testcase: &ioi::TestcaseInfo) -> Row {
    Row {
        content: RowContent::Data,
        cells: vec![
            Cell {
                content: CellContent::RowNumber(RowNumberCellContent {
                    number: testcase.id.try_into().expect("Testcase ID too large"),
                }),
            },
            Cell {
                content: CellContent::Score(ScoreCellContent {
                    range: Range {
                        precision: 2,
                        max: Score(1.),
                    },
                    r#ref: Key(format!("testcase.{}.score", testcase.id)),
                }),
            },
        ],
    }
}

fn files_in_dir(dir_path: &std::path::PathBuf) -> impl Iterator<Item = std::path::PathBuf> {
    std::fs::read_dir(dir_path)
        .expect("unable to read_dir")
        .map(|entry| entry.expect("unable to read_dir").path())
}

fn attachment_at_path(file_path: std::path::PathBuf) -> Attachment {
    Attachment {
        title: vec![TextVariant {
            attributes: vec![],
            value: file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        }],
        file: vec![FileVariant {
            attributes: vec![],
            name: Some(FileName(
                file_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            )),
            r#type: mime_guess::from_path(&file_path)
                .first_raw()
                .map(|t| MediaType(t.to_owned())),
            content: FileContent(std::fs::read(&file_path.to_string_lossy().as_ref()).unwrap()),
        }],
    }
}

/// Mapping (extension, MIME type)
const STATEMENT_FORMATS: [(&'static str, &'static str); 3] = [
    ("pdf", "application/pdf"),
    ("html", "text/html"),
    ("md", "application/markdown"),
];

/// Find the statements directory, as in the italy_yaml task format
/// Searches the paths $task_dir/statement and $task_dir/testo
fn statements_dir(task_dir: &Path) -> Option<PathBuf> {
    for dir in vec!["statement", "testo"] {
        let dir = task_dir.join(dir);
        if dir.exists() && dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

/// Tries to match the filename, returning the mimetype of the
/// matched item, if any
fn match_statement(path: &Path) -> Option<(String, FileName, MediaType)> {
    let ext = path.extension().unwrap().to_str().unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();
    let language = path.file_stem().unwrap().to_str().unwrap();
    for &(extension, mime_type) in &STATEMENT_FORMATS {
        if ext == extension {
            return Some((
                language.to_owned(),
                FileName(filename.to_owned()),
                MediaType(mime_type.to_owned()),
            ));
        }
    }
    None
}

/// find all the statements in the directory
fn statements_of(task_dir: &Path) -> Vec<FileVariant> {
    let mut result = Vec::new();
    let dir = statements_dir(task_dir);
    if let Some(dir) = dir {
        for file in dir.read_dir().unwrap() {
            let file = file.unwrap().path();
            if let Some((language, filename, mime_type)) = match_statement(&file) {
                result.push(FileVariant {
                    attributes: vec![VariantAttribute {
                        key: "language_name".to_owned(),
                        value: language,
                    }],
                    name: Some(filename),
                    r#type: Some(mime_type),
                    content: FileContent(
                        std::fs::read(file).expect("Unable to read statement file"),
                    ),
                });
            }
        }
    }
    result
}

pub fn gen_material(task: &ioi::Task) -> Material {
    Material {
        title: vec![TextVariant {
            attributes: vec![],
            value: task.title.clone().into(),
        }],
        statement: statements_of(&task.path),
        attachments: files_in_dir(&task.path.join("att"))
            .map(attachment_at_path)
            .collect(),
        submission_form: submission_form(),
        scorables: {
            subtasks_of(task)
                .into_iter()
                .filter(|s| s.max_score > 0.0)
                .map(scorable_of)
                .collect()
        },
        feedback: vec![Section::Table(TableSection {
            caption: caption(),
            cols: cols(),
            row_groups: subtasks_of(task).into_iter().map(row_group_of).collect(),
        })],
    }
}
