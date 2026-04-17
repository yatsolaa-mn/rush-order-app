use super::schema;
use shopify_function::prelude::*;
use shopify_function::Result;

fn rush_order_selected(
    line: &schema::run::rush_order_cart_transform_input::cart::Lines,
) -> bool {
    match line.rush_order().as_ref() {
        Some(a) => a
            .value()
            .as_deref()
            .is_some_and(|v| v.eq_ignore_ascii_case("true")),
        None => false,
    }
}

fn parse_percentage_value(raw: &str) -> Option<f64> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    let pct: f64 = s.parse().ok()?;
    if !pct.is_finite() || pct <= 0.0 {
        return None;
    }
    Some(pct)
}

fn rush_percentage_from_line(
    line: &schema::run::rush_order_cart_transform_input::cart::Lines,
) -> Option<f64> {
    use schema::run::rush_order_cart_transform_input::cart::lines::Merchandise;
    let pv = match line.merchandise() {
        Merchandise::ProductVariant(v) => v,
        _ => return None,
    };
    let product = pv.product();
    let rush_mf = product.rush_order_cost();
    let cost = rush_mf.as_ref()?;
    parse_percentage_value(cost.value())
}

fn unit_price_amount(line: &schema::run::rush_order_cart_transform_input::cart::Lines) -> Option<f64> {
    let amount = line.cost().amount_per_quantity().amount();
    let x = f64::from(*amount);
    if x.is_finite() && x > 0.0 {
        Some(x)
    } else {
        None
    }
}

#[shopify_function]
fn cart_transform_run(
    input: schema::run::RushOrderCartTransformInput,
) -> Result<schema::CartTransformRunResult> {
    let cart = input.cart();
    let mut operations = Vec::new();

    for line in cart.lines() {
        if !rush_order_selected(line) {
            continue;
        }
        let Some(pct) = rush_percentage_from_line(line) else {
            continue;
        };
        let Some(unit) = unit_price_amount(line) else {
            continue;
        };

        let multiplier = 1.0 + (pct / 100.0);
        if !multiplier.is_finite() || multiplier <= 0.0 {
            continue;
        }
        let new_unit = unit * multiplier;
        if !new_unit.is_finite() || new_unit <= 0.0 {
            continue;
        }

        let adjustment = schema::LineUpdateOperationFixedPricePerUnitAdjustment {
            amount: schema::Decimal::from(new_unit),
        };
        let price_value = schema::LineUpdateOperationPriceAdjustmentValue::FixedPricePerUnit(adjustment);
        let price = schema::LineUpdateOperationPriceAdjustment {
            adjustment: price_value,
        };

        operations.push(schema::Operation::LineUpdate(schema::LineUpdateOperation {
            cart_line_id: line.id().clone(),
            image: None,
            price: Some(price),
            title: None,
        }));
    }

    Ok(schema::CartTransformRunResult { operations })
}
