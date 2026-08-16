use phalcom_ast::ast::{ImportPath, ImportRoot, PathSegment};
use phalcom_modules::{
    FilesystemSourceProvider, ModuleComponent, ModuleId, ModuleKind, ModulePath, ModuleResolutionError, ModuleResolver, ProjectManifest, ProjectUniverse,
    ResolvedProjectId, SourceProvider, discover_owning_project,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_project_manifest_parsing_and_validation() {
    let toml = r#"
[project]
name = "my-awesome-lib"
version = "0.1.0"
authors = ["Test Author"]

[dependencies]
geometry = { path = "../geometry" }
"#;

    let manifest: ProjectManifest = toml::from_str(toml).expect("manifest should parse");
    let validated = manifest.validate().expect("manifest should validate");
    assert_eq!(validated.name.as_str(), "my_awesome_lib");
    assert_eq!(validated.raw_name, "my-awesome-lib");
    assert_eq!(validated.dependencies.len(), 1);
}

#[test]
fn test_project_universe_and_discovery() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create project structure
    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"app\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();

    let discovered = discover_owning_project(&proj_dir.join("src/main.ph")).expect("should discover project");
    let root_dir = discovered.expect("root dir found");
    assert_eq!(root_dir, proj_dir);

    let mut universe = ProjectUniverse::new();
    let root_id = universe.load_root(root_dir.join("project.toml")).expect("universe load succeeds");
    assert_eq!(root_id, ResolvedProjectId::from_raw(1));
    assert_eq!(universe.projects().len(), 1);
}

#[test]
fn test_filesystem_source_provider_resolution() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("geometry");
    fs::create_dir_all(proj_dir.join("src/shapes")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"geometry\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "expose .shapes\n").unwrap();
    fs::write(proj_dir.join("src/shapes/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/point.ph"), "class Point {}\nexport Point\n").unwrap();
    fs::write(proj_dir.join("src/shapes/circle.ph"), "class Circle {}\nexport Circle\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let root_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(root_id).unwrap();

    let provider = FilesystemSourceProvider::new();

    // 1. Locate package root
    let root_unit = provider.locate(project, &ModulePath::root()).unwrap();
    assert_eq!(root_unit.kind, ModuleKind::Package);

    // 2. Locate point module
    let point_path = ModulePath::from_components(vec![ModuleComponent::from_identifier("point").unwrap()]);
    let point_unit = provider.locate(project, &point_path).unwrap();
    assert_eq!(point_unit.kind, ModuleKind::Module);

    // 3. Locate shapes.circle module
    let circle_path = ModulePath::from_components(vec![
        ModuleComponent::from_identifier("shapes").unwrap(),
        ModuleComponent::from_identifier("circle").unwrap(),
    ]);
    let circle_unit = provider.locate(project, &circle_path).unwrap();
    assert_eq!(circle_unit.kind, ModuleKind::Module);
}

#[test]
fn test_module_resolver_logical_imports_and_exposure() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create library project: geometry
    let lib_dir = root.join("geometry");
    fs::create_dir_all(lib_dir.join("src/shapes")).unwrap();
    fs::write(lib_dir.join("project.toml"), "[project]\nname = \"geometry\"\nversion = \"0.1.0\"\n").unwrap();
    // Exposes .point and .shapes (which exposes .circle)
    fs::write(lib_dir.join("src/package.ph"), "expose .point\nexpose .shapes\n").unwrap();
    fs::write(lib_dir.join("src/point.ph"), "class Point {}\nexport Point\n").unwrap();
    fs::write(lib_dir.join("src/shapes/package.ph"), "expose .circle\n").unwrap();
    fs::write(lib_dir.join("src/shapes/circle.ph"), "class Circle {}\nexport Circle\n").unwrap();
    fs::write(lib_dir.join("src/private_tool.ph"), "class Secret {}\n").unwrap();

    // Create app project: app depending on geometry
    let app_dir = root.join("app");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("project.toml"),
        "[project]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ngeometry = { path = \"../geometry\" }\n",
    )
    .unwrap();
    fs::write(app_dir.join("src/package.ph"), "").unwrap();
    fs::write(app_dir.join("src/main.ph"), "import geometry.point\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let app_root_id = universe.load_root(app_dir.join("project.toml")).unwrap();
    let source_provider = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &source_provider);

    let app_main_id = ModuleId {
        project: app_root_id,
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    };

    // 1. Resolve exposed cross-project import: `geometry.point`
    let import_point = ImportPath {
        root: ImportRoot::Absolute(PathSegment {
            name: "geometry".to_string(),
            range: (0..8).into(),
        }),
        segments: vec![PathSegment {
            name: "point".to_string(),
            range: (9..14).into(),
        }],
        range: (0..14).into(),
    };
    let resolved_point = resolver.resolve_import(&app_main_id, &import_point).expect("geometry.point should resolve");
    assert_eq!(resolved_point.kind, ModuleKind::Module);

    // 2. Resolve hierarchical exposed import: `geometry.shapes.circle`
    let import_circle = ImportPath {
        root: ImportRoot::Absolute(PathSegment {
            name: "geometry".to_string(),
            range: (0..8).into(),
        }),
        segments: vec![
            PathSegment {
                name: "shapes".to_string(),
                range: (9..15).into(),
            },
            PathSegment {
                name: "circle".to_string(),
                range: (16..22).into(),
            },
        ],
        range: (0..22).into(),
    };
    let resolved_circle = resolver
        .resolve_import(&app_main_id, &import_circle)
        .expect("geometry.shapes.circle should resolve");
    assert_eq!(resolved_circle.kind, ModuleKind::Module);

    // 3. Resolve unexposed cross-project import: `geometry.private_tool` -> Expect ModulePathNotExposed error
    let import_secret = ImportPath {
        root: ImportRoot::Absolute(PathSegment {
            name: "geometry".to_string(),
            range: (0..8).into(),
        }),
        segments: vec![PathSegment {
            name: "private_tool".to_string(),
            range: (9..21).into(),
        }],
        range: (0..21).into(),
    };
    let err = resolver.resolve_import(&app_main_id, &import_secret).unwrap_err();
    assert!(matches!(err, ModuleResolutionError::ModulePathNotExposed { .. }));
}
