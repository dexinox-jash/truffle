//! Test Data Fixtures
//!
//! Pre-built test data sets for integration tests including:
//! - Sample organizations and people
//! - Complete graph structures
//! - Sample facts for validation testing

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use truffle_core::{
    Entity, EntityType, EntityMetadata, PrivacyLevel,
    Relationship,
    validation::{Fact, FactValue, FactSource},
};

use crate::integration::{TestEntityBuilder, TestRelationshipBuilder, TestFactBuilder};

/// A complete test dataset containing entities, relationships, and facts
pub struct TestDataset {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub facts: Vec<Fact>,
    pub entity_map: HashMap<String, Uuid>,
}

impl TestDataset {
    /// Create an empty dataset
    pub fn empty() -> Self {
        Self {
            entities: Vec::new(),
            relationships: Vec::new(),
            facts: Vec::new(),
            entity_map: HashMap::new(),
        }
    }

    /// Get entity by name
    pub fn get_entity(&self, name: &str) -> Option<&Entity> {
        self.entity_map
            .get(name)
            .and_then(|id| self.entities.iter().find(|e| e.id == *id))
    }

    /// Get entity ID by name
    pub fn get_entity_id(&self, name: &str) -> Option<Uuid> {
        self.entity_map.get(name).copied()
    }

    /// Get relationships for an entity
    pub fn get_relationships_for(&self, entity_id: Uuid) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|r| r.source_id == entity_id || r.target_id == entity_id)
            .collect()
    }

    /// Get facts for an entity
    pub fn get_facts_for(&self, entity_id: Uuid) -> Vec<&Fact> {
        self.facts
            .iter()
            .filter(|f| f.entity_id == entity_id)
            .collect()
    }
}

/// Create a tech company dataset with employees, departments, and locations
pub fn tech_company_dataset() -> TestDataset {
    let mut dataset = TestDataset::empty();

    // Create the main company
    let acme_corp = TestEntityBuilder::organization("Acme Corp")
        .with_industry("Technology")
        .with_description("A leading technology company")
        .verified()
        .build();
    dataset.entity_map.insert("Acme Corp".to_string(), acme_corp.id);
    dataset.entities.push(acme_corp.clone());

    // Create departments
    let departments = vec![
        ("Engineering", "Software development and infrastructure"),
        ("Sales", "Revenue and customer acquisition"),
        ("Marketing", "Brand and growth marketing"),
        ("HR", "People and culture"),
    ];

    for (name, desc) in departments {
        let dept = TestEntityBuilder::organization(&format!("{} Department", name))
            .with_description(desc)
            .build();
        dataset.entity_map.insert(name.to_string(), dept.id);
        dataset.entities.push(dept.clone());

        // Link department to company
        let rel = TestRelationshipBuilder::new(dept.id, acme_corp.id, "part_of")
            .build();
        dataset.relationships.push(rel);
    }

    // Create locations
    let locations = vec![
        ("San Francisco HQ", 37.7749, -122.4194, "Primary headquarters"),
        ("New York Office", 40.7128, -74.0060, "East coast office"),
        ("London Office", 51.5074, -0.1278, "European headquarters"),
    ];

    for (name, lat, lon, desc) in locations {
        let location = TestEntityBuilder::location(name, lat, lon)
            .with_description(desc)
            .build();
        dataset.entity_map.insert(name.to_string(), location.id);
        dataset.entities.push(location.clone());

        // Link location to company
        let rel = TestRelationshipBuilder::located_in(acme_corp.id, location.id)
            .build();
        dataset.relationships.push(rel);
    }

    // Create employees
    let employees = vec![
        ("Alice Johnson", "CEO", "alice@acme.com", "Engineering"),
        ("Bob Smith", "CTO", "bob@acme.com", "Engineering"),
        ("Carol White", "VP Engineering", "carol@acme.com", "Engineering"),
        ("David Brown", "Senior Engineer", "david@acme.com", "Engineering"),
        ("Eve Davis", "Engineer", "eve@acme.com", "Engineering"),
        ("Frank Miller", "VP Sales", "frank@acme.com", "Sales"),
        ("Grace Lee", "Sales Director", "grace@acme.com", "Sales"),
        ("Henry Wilson", "Sales Rep", "henry@acme.com", "Sales"),
        ("Ivy Chen", "VP Marketing", "ivy@acme.com", "Marketing"),
        ("Jack Taylor", "Marketing Manager", "jack@acme.com", "Marketing"),
    ];

    for (name, title, email, dept_name) in employees {
        let person = TestEntityBuilder::person(name)
            .with_role(title)
            .with_email(email)
            .build();
        dataset.entity_map.insert(name.to_string(), person.id);
        dataset.entities.push(person.clone());

        // Link to company
        let works_at = TestRelationshipBuilder::works_at(person.id, acme_corp.id)
            .build();
        dataset.relationships.push(works_at);

        // Link to department
        if let Some(dept_id) = dataset.get_entity_id(dept_name) {
            let in_dept = TestRelationshipBuilder::new(person.id, dept_id, "member_of")
                .build();
            dataset.relationships.push(in_dept);
        }
    }

    // Create reporting relationships
    let reporting = vec![
        ("Bob Smith", "Alice Johnson"),      // CTO reports to CEO
        ("Carol White", "Bob Smith"),        // VP Eng reports to CTO
        ("David Brown", "Carol White"),      // Sr Eng reports to VP Eng
        ("Eve Davis", "Carol White"),        // Eng reports to VP Eng
        ("Frank Miller", "Alice Johnson"),   // VP Sales reports to CEO
        ("Grace Lee", "Frank Miller"),       // Sales Dir reports to VP Sales
        ("Henry Wilson", "Grace Lee"),       // Sales Rep reports to Sales Dir
        ("Ivy Chen", "Alice Johnson"),       // VP Marketing reports to CEO
        ("Jack Taylor", "Ivy Chen"),         // Marketing Mgr reports to VP Marketing
    ];

    for (employee, manager) in reporting {
        if let (Some(emp_id), Some(mgr_id)) = (
            dataset.get_entity_id(employee),
            dataset.get_entity_id(manager),
        ) {
            let reports_to = TestRelationshipBuilder::new(emp_id, mgr_id, "reports_to")
                .build();
            dataset.relationships.push(reports_to);
        }
    }

    // Add some "knows" relationships
    let knows_relationships = vec![
        ("David Brown", "Eve Davis"),
        ("Grace Lee", "Henry Wilson"),
        ("Ivy Chen", "Jack Taylor"),
        ("Bob Smith", "Carol White"),
    ];

    for (a, b) in knows_relationships {
        if let (Some(id_a), Some(id_b)) = (dataset.get_entity_id(a), dataset.get_entity_id(b)) {
            let knows = TestRelationshipBuilder::knows(id_a, id_b).build();
            dataset.relationships.push(knows);
        }
    }

    dataset
}

/// Create a dataset with temporal facts for validation testing
pub fn temporal_facts_dataset() -> TestDataset {
    let mut dataset = TestDataset::empty();
    let now = Utc::now();

    // Create a person with employment history
    let person = TestEntityBuilder::person("John Doe")
        .with_email("john.doe@example.com")
        .build();
    dataset.entity_map.insert("John Doe".to_string(), person.id);
    dataset.entities.push(person.clone());

    // Create companies
    let companies = vec![
        ("Startup Inc", now - Duration::days(1095), now - Duration::days(730)),
        ("Growth Co", now - Duration::days(700), now - Duration::days(365)),
        ("Big Corp", now - Duration::days(300), None), // Current
    ];

    for (name, start, end) in companies {
        let company = TestEntityBuilder::organization(name)
            .build();
        dataset.entity_map.insert(name.to_string(), company.id);
        dataset.entities.push(company.clone());

        // Create employment relationship
        let rel = TestRelationshipBuilder::works_at(person.id, company.id)
            .with_validity(start, end)
            .build();
        dataset.relationships.push(rel);

        // Create employment fact
        let fact = TestFactBuilder::employer(person.id, name)
            .with_valid_range(Some(start), end)
            .with_confidence(0.9)
            .build();
        dataset.facts.push(fact);
    }

    // Add title progression
    let titles = vec![
        ("Junior Developer", now - Duration::days(1095), now - Duration::days(730)),
        ("Developer", now - Duration::days(700), now - Duration::days(365)),
        ("Senior Developer", now - Duration::days(300), None),
    ];

    for (title, start, end) in titles {
        let fact = TestFactBuilder::title(person.id, title)
            .with_valid_range(Some(start), end)
            .with_confidence(0.85)
            .build();
        dataset.facts.push(fact);
    }

    dataset
}

/// Create a dataset with potential duplicates for deduplication testing
pub fn duplicates_dataset() -> TestDataset {
    let mut dataset = TestDataset::empty();

    // Create entities with similar names (potential duplicates)
    let similar_people = vec![
        ("John Smith", "john.smith@example.com", "Engineer"),
        ("Jon Smith", "jon.smith@example.com", "Engineer"),       // Typo
        ("John A. Smith", "john.a.smith@example.com", "Engineer"), // Middle initial
        ("John Smith", "john.smith@example.com", "Developer"),     // Exact duplicate
    ];

    for (i, (name, email, role)) in similar_people.iter().enumerate() {
        let person = TestEntityBuilder::person(&format!("{} {}", name, i))
            .with_email(email)
            .with_role(role)
            .build();
        dataset.entity_map.insert(format!("person_{}", i), person.id);
        dataset.entities.push(person);
    }

    // Create entities with same email (definite duplicates)
    let same_email = vec![
        ("Robert Johnson", "bob@company.com"),
        ("Bob Johnson", "bob@company.com"),     // Same email
        ("R. Johnson", "bob@company.com"),      // Same email, different name
    ];

    for (i, (name, email)) in same_email.iter().enumerate() {
        let person = TestEntityBuilder::person(name)
            .with_email(email)
            .build();
        dataset.entity_map.insert(format!("same_email_{}", i), person.id);
        dataset.entities.push(person);
    }

    // Create similar organizations
    let similar_orgs = vec![
        ("Acme Corporation", "acme.com"),
        ("Acme Corp", "acme.com"),              // Shortened name, same domain
        ("ACME Corp.", "acme.com"),             // Different capitalization
        ("Acme Inc", "acmeinc.com"),            // Different name, different domain
    ];

    for (i, (name, domain)) in similar_orgs.iter().enumerate() {
        let org = TestEntityBuilder::organization(name)
            .build();
        dataset.entity_map.insert(format!("org_{}", i), org.id);
        dataset.entities.push(org);
    }

    dataset
}

/// Create a dataset with contradictions for validation testing
pub fn contradictions_dataset() -> TestDataset {
    let mut dataset = TestDataset::empty();
    let now = Utc::now();

    // Create a person with conflicting facts
    let person = TestEntityBuilder::person("Jane Doe")
        .with_email("jane@example.com")
        .build();
    dataset.entity_map.insert("Jane Doe".to_string(), person.id);
    dataset.entities.push(person.clone());

    // Temporal contradiction: overlapping employers
    dataset.facts.push(
        TestFactBuilder::employer(person.id, "Company A")
            .with_valid_range(Some(now - Duration::days(180)), Some(now + Duration::days(30)))
            .with_confidence(0.8)
            .from_extraction(Uuid::new_v4())
            .build()
    );

    dataset.facts.push(
        TestFactBuilder::employer(person.id, "Company B")
            .with_valid_range(Some(now - Duration::days(90)), Some(now + Duration::days(60)))
            .with_confidence(0.85)
            .from_extraction(Uuid::new_v4())
            .build()
    );

    // Factual contradiction: different departments (both claimed current)
    let verified_user = Uuid::new_v4();
    dataset.facts.push(
        TestFactBuilder::new(person.id, "department", FactValue::text("Engineering"))
            .verified(verified_user)
            .build()
    );

    dataset.facts.push(
        TestFactBuilder::new(person.id, "department", FactValue::text("Sales"))
            .with_confidence(0.7)
            .from_extraction(Uuid::new_v4())
            .build()
    );

    // Location contradiction
    dataset.facts.push(
        TestFactBuilder::location(person.id, "San Francisco")
            .verified(verified_user)
            .build()
    );

    dataset.facts.push(
        TestFactBuilder::location(person.id, "New York")
            .with_confidence(0.6)
            .from_extraction(Uuid::new_v4())
            .build()
    );

    dataset
}

/// Create a social network dataset
pub fn social_network_dataset(num_people: usize, connection_density: f32) -> TestDataset {
    let mut dataset = TestDataset::empty();
    let mut rng = SimpleRng::new(42);

    // Create people
    let first_names = vec![
        "Alice", "Bob", "Carol", "David", "Eve", "Frank", "Grace", "Henry",
        "Ivy", "Jack", "Kate", "Liam", "Mia", "Noah", "Olivia", "Paul",
    ];
    let last_names = vec![
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
        "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas",
    ];

    for i in 0..num_people {
        let first = first_names[rng.next() as usize % first_names.len()];
        let last = last_names[rng.next() as usize % last_names.len()];
        let name = format!("{} {}", first, last);

        let person = TestEntityBuilder::person(&name)
            .build();
        dataset.entity_map.insert(format!("person_{}", i), person.id);
        dataset.entities.push(person);
    }

    // Create "knows" relationships based on density
    let num_connections = (num_people as f32 * connection_density) as usize;

    for _ in 0..num_connections {
        let i = (rng.next() as usize) % num_people;
        let j = (rng.next() as usize) % num_people;

        if i != j {
            if let (Some(id_i), Some(id_j)) = (
                dataset.get_entity_id(&format!("person_{}", i)),
                dataset.get_entity_id(&format!("person_{}", j)),
            ) {
                let rel = TestRelationshipBuilder::knows(id_i, id_j).build();
                dataset.relationships.push(rel);
            }
        }
    }

    dataset
}

/// Create a dataset for product catalog testing
pub fn product_catalog_dataset() -> TestDataset {
    let mut dataset = TestDataset::empty();

    // Create vendor company
    let vendor = TestEntityBuilder::organization("TechVendor Inc")
        .with_industry("Software")
        .build();
    dataset.entity_map.insert("TechVendor".to_string(), vendor.id);
    dataset.entities.push(vendor.clone());

    // Create product categories as concepts
    let categories = vec![
        ("Cloud Computing", "Infrastructure and platform services"),
        ("AI/ML Tools", "Machine learning and artificial intelligence"),
        ("Security", "Cybersecurity and compliance"),
        ("Analytics", "Data analytics and visualization"),
    ];

    for (name, desc) in categories {
        let concept = TestEntityBuilder::concept(name)
            .with_description(desc)
            .build();
        dataset.entity_map.insert(name.to_string(), concept.id);
        dataset.entities.push(concept.clone());
    }

    // Create products
    let products = vec![
        ("CloudCompute Pro", "Cloud Computing", "$99/month", "Enterprise cloud infrastructure"),
        ("SecureVault", "Security", "$49/month", "Enterprise password management"),
        ("DataInsights", "Analytics", "$199/month", "Business intelligence platform"),
        ("ML Studio", "AI/ML Tools", "$299/month", "Machine learning workbench"),
        ("CloudCompute Basic", "Cloud Computing", "$29/month", "Starter cloud package"),
    ];

    for (name, category, price, desc) in products {
        let product = TestEntityBuilder::product(name)
            .with_description(desc)
            .with_metadata(EntityMetadata {
                category: Some(category.to_string()),
                price: Some(price.to_string()),
                ..Default::default()
            })
            .build();
        dataset.entity_map.insert(name.to_string(), product.id);
        dataset.entities.push(product.clone());

        // Link to vendor
        let rel = TestRelationshipBuilder::new(product.id, vendor.id, "produced_by")
            .build();
        dataset.relationships.push(rel);

        // Link to category
        if let Some(cat_id) = dataset.get_entity_id(category) {
            let cat_rel = TestRelationshipBuilder::new(product.id, cat_id, "in_category")
                .build();
            dataset.relationships.push(cat_rel);
        }
    }

    dataset
}

/// Simple deterministic RNG for reproducible test data
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
}

/// Predefined fixtures for common test scenarios
pub mod fixtures {
    use super::*;

    /// Small company with 10 employees
    pub fn small_company() -> TestDataset {
        tech_company_dataset()
    }

    /// Medium social network with 50 people
    pub fn medium_social_network() -> TestDataset {
        social_network_dataset(50, 2.0)
    }

    /// Large social network with 200 people
    pub fn large_social_network() -> TestDataset {
        social_network_dataset(200, 3.0)
    }

    /// Dataset with clear duplicate candidates
    pub fn duplicate_candidates() -> TestDataset {
        duplicates_dataset()
    }

    /// Dataset with various contradictions
    pub fn contradiction_examples() -> TestDataset {
        contradictions_dataset()
    }

    /// Dataset with temporal employment history
    pub fn employment_history() -> TestDataset {
        temporal_facts_dataset()
    }

    /// Product catalog for e-commerce testing
    pub fn product_catalog() -> TestDataset {
        product_catalog_dataset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tech_company_dataset() {
        let dataset = tech_company_dataset();

        // Should have company + 4 departments + 3 locations + 10 employees
        assert!(dataset.entities.len() >= 18);
        assert!(!dataset.relationships.is_empty());

        // Check entity lookup
        let acme = dataset.get_entity("Acme Corp");
        assert!(acme.is_some());
        assert_eq!(acme.unwrap().entity_type, EntityType::Organization);

        let alice = dataset.get_entity("Alice Johnson");
        assert!(alice.is_some());
        assert_eq!(alice.unwrap().entity_type, EntityType::Person);
    }

    #[test]
    fn test_temporal_facts_dataset() {
        let dataset = temporal_facts_dataset();

        assert!(!dataset.entities.is_empty());
        assert!(!dataset.facts.is_empty());

        // Check facts exist
        let john_id = dataset.get_entity_id("John Doe").unwrap();
        let facts = dataset.get_facts_for(john_id);
        assert!(!facts.is_empty());
    }

    #[test]
    fn test_duplicates_dataset() {
        let dataset = duplicates_dataset();

        assert!(!dataset.entities.is_empty());

        // Should have potential duplicates
        let similar_count = dataset.entities
            .iter()
            .filter(|e| e.name.contains("John") || e.name.contains("Jon"))
            .count();
        assert!(similar_count >= 2);
    }

    #[test]
    fn test_contradictions_dataset() {
        let dataset = contradictions_dataset();

        assert!(!dataset.entities.is_empty());
        assert!(!dataset.facts.is_empty());

        // Should have multiple facts for the same person
        let jane_id = dataset.get_entity_id("Jane Doe").unwrap();
        let facts = dataset.get_facts_for(jane_id);
        assert!(facts.len() > 2);
    }

    #[test]
    fn test_social_network_dataset() {
        let dataset = social_network_dataset(20, 1.5);

        assert_eq!(dataset.entities.len(), 20);
        // With density 1.5, should have approximately 30 connections
        assert!(!dataset.relationships.is_empty());
    }

    #[test]
    fn test_product_catalog_dataset() {
        let dataset = product_catalog_dataset();

        assert!(!dataset.entities.is_empty());

        let vendor = dataset.get_entity("TechVendor Inc");
        assert!(vendor.is_some());

        let product = dataset.get_entity("CloudCompute Pro");
        assert!(product.is_some());
    }

    #[test]
    fn test_simple_rng() {
        let mut rng1 = SimpleRng::new(42);
        let mut rng2 = SimpleRng::new(42);

        // Same seed should produce same sequence
        assert_eq!(rng1.next(), rng2.next());
        assert_eq!(rng1.next(), rng2.next());

        // Different seeds should diverge
        let mut rng3 = SimpleRng::new(100);
        assert_ne!(rng1.next(), rng3.next());
    }
}
