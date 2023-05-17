sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "sap/m/MessageToast"
], function (BaseController, Filter, FilterOperator, MessageToast) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ScoringTable", {

    onInit: async function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oTable = this.getView().byId("scoringTable");

      this._oScoringModel = await this.getJSONModel(`/api/regattas/${this.getRegattaId()}/scoring`, this._oTable);
      this.setViewModel(this._oScoringModel, "scoring");

      this.getRouter().getRoute("scoring").attachMatched(async (_) => await this._loadScoringModel(), this);
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
              new Filter("club/longName", FilterOperator.Contains, sQuery),
              new Filter("club/city", FilterOperator.Contains, sQuery)
            ],
            and: false
          }))
      }
      const oBinding = this._oTable.getBinding("items");
      oBinding.filter(aSearchFilters);
    },

    onRefreshButtonPress: async function (oEvent) {
      await this._loadScoringModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    },

    _loadScoringModel: async function () {
      await this.updateJSONModel(this._oScoringModel, `/api/regattas/${this.getRegattaId()}/scoring`, this._oTable)
    }

  });
});