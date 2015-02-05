require 'net/http'
require 'json'

# In order to make this work locally
# 1. Start server
# 2. Add path to csv data to command line arguments (for consistent dummy data)
# 3. Run tests
# 4. Stop server

def get(path)
  uri = URI("http://localhost:3000#{path}")
  res = Net::HTTP.get(uri)
  JSON.parse(res)
end

describe "competitors" do
  describe "single" do
    before(:all) do
      @json_response = get("/competitors/2007HABE01")
    end

    let(:competitor) { @json_response["competitor"] }

    it "returns the competition count" do
      expect(competitor["competition_count"]).to eq 36
    end

    it "returns the name" do
      expect(competitor["name"]).to eq "Tim Habermaas"
    end

    it "returns the gender" do
      expect(competitor["gender"]).to eq "m"
    end

    it "returns the id" do
      expect(competitor["id"]).to eq "2007HABE01"
    end

    it "returns the country" do
      expect(competitor["country"]).to eq "Germany"
    end
  end

  describe "searching" do
    before(:all) do
      @json_response = get("/competitors?q=2009BAC")
    end

    let(:competitors) { @json_response["competitors"] }

    it "returns 2 results" do
      expect(competitors.size).to eq 4
      expect(competitors.first["name"]).to eq "Benoit Bacher"
      expect(competitors.first["country"]).to eq "France"
    end
  end
end
