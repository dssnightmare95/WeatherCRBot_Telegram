use reqwest::Client;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
struct Coordinates {
    lat: f64,
    lng: f64,
}
impl Coordinates {
    fn new(lat: f64, lng: f64) -> Self {
        Coordinates { lat, lng }
    }
}

#[derive(Debug, Clone)]
struct Weather {
    weather_condition: String,
    temperature: f64,
    feels_like: f64,
    humidity: f64,
    precipitation_probability: f64,
}
impl Weather {
    fn new(weather_condition: String, temperature: f64, feels_like: f64, humidity: f64, precipitation_probability: f64) -> Self {
        Weather { weather_condition, temperature, feels_like, humidity, precipitation_probability }
    }
}

async fn get_coordinates(location: String, province: String, country: String, api_key: String) -> Result<Coordinates, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://maps.googleapis.com/maps/api/geocode/json?address={}+{}+{}&key={}",
        location.replace(" ", "+"), province.replace(" ", "+"), country.replace(" ", "+"), api_key
    );
    let client = Client::new();
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if response["status"] == "ZERO_RESULTS" {
        return Err("No results found".into());
    }

    let coordinates = Coordinates::new(
        response["results"][0]["geometry"]["location"]["lat"].as_f64().unwrap(),
        response["results"][0]["geometry"]["location"]["lng"].as_f64().unwrap(),
    );

    Ok(coordinates)
}

async fn get_weather(location_coord: Coordinates, api_key: String) -> Result<Weather, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://weather.googleapis.com/v1/currentConditions:lookup?key={}&location.latitude={}&location.longitude={}",
        api_key, location_coord.lat, location_coord.lng
    );
    let client = Client::new();
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<Value>()
        .await?;
    let weather = Weather::new(
        response["weatherCondition"]["description"]["text"].as_str().unwrap().to_string(),
        response["temperature"]["degrees"].as_f64().unwrap(),
        response["feelsLikeTemperature"]["degrees"].as_f64().unwrap(),
        response["relativeHumidity"].as_f64().unwrap(),
        response["precipitation"]["probability"]["percent"].as_f64().unwrap(),
    );
    Ok(weather)
} 


async fn get_inverted_coordinates(coordinates: Coordinates, api_key: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://maps.googleapis.com/maps/api/geocode/json?latlng={},{}&key={}",
        coordinates.lat.clone().to_string(), coordinates.lng.clone().to_string(), api_key
    );
    let client = Client::new();
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<Value>()
        .await?;


    Ok(response["results"][2]["formatted_address"].as_str().unwrap().to_string())
}

pub async fn get_weater_information(location: String, province: String, country: String, google_api_key: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match get_coordinates(location.clone(), province.clone(), country.clone(), google_api_key.clone()).await {
        Ok(coordinates_data) => {
            let weather_data = get_weather(coordinates_data, google_api_key.clone()).await?;

            let weather_info = format!(
                "🌤️ The weather in {}, {}, {} is\nWeather Condition: {}\nTemperature: {}°C\nFeels Like: {}°C\nHumidity: {}%\nPrecipitation Probability: {}%",
                location,
                province,
                country,
                weather_data.weather_condition,
                weather_data.temperature,
                weather_data.feels_like,
                weather_data.humidity,
                weather_data.precipitation_probability
            );
            Ok(weather_info)
        }
        Err(_e) => {
            let weather_info = format!("❌ Please check the location, province, and country you provided.");
            Ok(weather_info)

        }
    }
}

pub async fn get_weater_information_from_location(lat: f64, lng: f64, google_api_key: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let coordinates_data = Coordinates::new(lat, lng);
    
    let location_name = get_inverted_coordinates(coordinates_data, google_api_key.clone()).await?;
    let weather_data = get_weather(coordinates_data, google_api_key.clone()).await?;

    let weather_info = format!(
        "🌤️ The weather in {} is\nWeather Condition: {}\nTemperature: {}°C\nFeels Like: {}°C\nHumidity: {}%\nPrecipitation Probability: {}%",
        location_name,
        weather_data.weather_condition,
        weather_data.temperature,
        weather_data.feels_like,
        weather_data.humidity,
        weather_data.precipitation_probability
    );
    
    Ok(weather_info)

}