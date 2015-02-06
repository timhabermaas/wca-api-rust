require 'net/http'
require 'json'

def get(path)
  uri = URI("http://localhost:3000#{path}")
  res = Net::HTTP.get(uri)
  JSON.parse(res)
end

describe "records" do
  describe "single" do
    before(:all) do
      @json_response = get("/records/333mbf/single")
    end

    let(:records) { @json_response }

    it "returns all records" do
      expect(records.size).to eq 926
      expect(records.first["time"]).to eq 580325400
      expect(records.first["competitor"]["name"]).to eq "Marcin Kowalczyk"
      expect(records.last["time"]).to eq 990360006
      expect(records.last["competitor"]["name"]).to eq"Rodson Lingad"
    end
  end

  describe "comparison" do
    before(:all) do
      @json_response = get("/records/333?ids=2007HABE01&ids=2003POCH01")
    end

    it "returns two records each containing single and average" do
      expect(@json_response.size).to eq 2
      expect(@json_response.first["competitor_id"]).to eq "2003POCH01"
      expect(@json_response.first["single"]["time"]).to eq 956
      expect(@json_response.first["average"]["time"]).to eq 1273
      expect(@json_response.last["competitor_id"]).to eq "2007HABE01"
      expect(@json_response.last["single"]["time"]).to eq 1087
      expect(@json_response.last["average"]["time"]).to eq 1376
    end
  end

  describe "records of a competitor" do
    before(:all) do
      @json_response = get("/competitors/2007HABE01/records")
    end

    it "returns all records" do
      expect(@json_response.size).to eq 16
      # TODO what a stupid response...
      expect(@json_response.first[0]).to eq "333oh"
      expect(@json_response.first[1]["single"]["time"]).to eq 2750
      expect(@json_response.first[1]["average"]["time"]).to eq 3067
    end
  end
end
