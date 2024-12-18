// Fetch token information
async function fetchTokenInfo() {
  try {
    const response = await fetch("https://api.dexie.space/v1/tokens");
    const { tokens } = await response.json();
    return tokens;
  } catch (error) {
    console.error("Error fetching token information:", error);
    return null;
  }
}

function updateElement(id, value) {
  document.getElementById(id).textContent = value;
}

function updateTokenIcon(id, tokenId) {
  document.getElementById(id).src = `https://icons.dexie.space/${tokenId}.webp`;
}

(async () => {
  const tokens = await fetchTokenInfo();
  const params = new URLSearchParams(window.location.search);

  const underlyingToken = tokens.find(
    (t) => t.id === params.get("underlying_asset")
  );
  const settlementToken = tokens.find(
    (t) => t.id === params.get("settlement_asset")
  );

  const underlyingAmount =
    params.get("underlying_mojos") / (underlyingToken?.denom || 1);
  const strikePrice =
    params.get("settlement_mojos") /
    (settlementToken?.denom || 1) /
    underlyingAmount;

  const expDate = new Date(params.get("expiration") * 1000)
    .toISOString()
    .replace("T", " ")
    .replace(/\.\d+Z$/, " UTC");

  // Update DOM
  updateTokenIcon("settlement_asset_icon", params.get("settlement_asset"));
  updateTokenIcon("underlying_asset_icon", params.get("underlying_asset"));
  updateTokenIcon("underlying_title_icon", params.get("underlying_asset"));

  updateElement("underlying_asset_amount", underlyingAmount);
  updateElement("strike_price", strikePrice);
  updateElement("underlying_title", underlyingToken?.code || "Unknown");
  updateElement("expiration", expDate);
  updateElement("contract_id", params.get("contract_id"));

  updateElement(
    "settlement_asset_code",
    settlementToken?.code || params.get("settlement_asset")
  );
  updateElement(
    "underlying_asset_code",
    underlyingToken?.code || params.get("underlying_asset")
  );

  new QRCode(document.getElementById("qrcode"), {
    text: params.get("contract_id"),
    width: 100,
    height: 100,
    colorDark: "#000000",
    colorLight: "#ffffff",
    correctLevel: QRCode.CorrectLevel.L,
  });
})();
