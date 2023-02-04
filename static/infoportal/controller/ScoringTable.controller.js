sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator"
], function (BaseController, Filter, FilterOperator) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ScoringTable", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("scoring").attachMatched(this._loadScoringModel, this);

      this.oTable = this.getView().byId("scoringTable");
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    onFilterSearch: function (oEvent) {
      const aSearchFilters = [];
      const sQuery = oEvent.getParameter("query").trim();
      if (sQuery) {
        aSearchFilters.push(
          new Filter({
            filters: [
              new Filter("club/shortName", FilterOperator.Contains, sQuery),
              new Filter("club/city", FilterOperator.Contains, sQuery)
            ],
            and: false
          }))
      }
      const oBinding = this.oTable.getBinding("items");
      oBinding.filter(aSearchFilters);
    },

    onRefreshButtonPress: async function (oEvent) {
      await this._loadScoringModel();
    },

    _loadScoringModel: async function () {
      if (!this._oScoringModel) {
        this._oScoringModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/scoring", this.oTable);
        this.setViewModel(this._oScoringModel, "scoring");
      } else {
        await this.updateJSONModel(this._oScoringModel, "/api/regattas/" + this.getRegattaId() + "/scoring", this.oTable)
      }
    }

  });
});